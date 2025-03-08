use crate::synxit::user::{AuthSession, Session, User};
use crate::utils::{current_time, random_u128, u128_to_32_char_hex_string};

impl User {
    pub fn create_session(&mut self) -> String {
        let id: u128 = random_u128();
        self.sessions.push(Session {
            id,
            created_at: current_time(),
            last_used: current_time(),
            root: false,
        });
        u128_to_32_char_hex_string(id)
    }

    pub fn create_auth_session(&mut self) -> String {
        let id: u128 = random_u128();
        self.auth.auth_sessions.push(AuthSession {
            id,
            expires_at: current_time() + 3600,
            challenge: random_u128(),
            mfa_count: 0,
            password_correct: false,
        });
        u128_to_32_char_hex_string(id)
    }

    pub fn get_session_by_id(&self, id: &str) -> &Session {
        let id = u128::from_str_radix(id, 16).expect("Failed to parse session id");
        self.sessions
            .iter()
            .find(|s| s.id == id)
            .expect("Failed to find session")
    }

    pub fn get_auth_session_by_id(&self, id: &str) -> &AuthSession {
        let id = u128::from_str_radix(id, 16).expect("Failed to parse auth session id");
        self.auth
            .auth_sessions
            .iter()
            .find(|s| s.id == id)
            .expect("Failed to find auth session")
    }

    fn get_mut_auth_session_by_id(&mut self, id: &str) -> &mut AuthSession {
        let id = u128::from_str_radix(id, 16).expect("Failed to parse auth session id");
        self.auth
            .auth_sessions
            .iter_mut()
            .find(|s| s.id == id)
            .expect("Failed to find auth session")
    }

    pub fn delete_auth_session_by_id(&mut self, id: &str) {
        let id = u128::from_str_radix(id, 16).expect("Failed to parse auth session id");
        self.auth.auth_sessions.retain(|s| s.id != id);
    }

    pub fn delete_session_by_id(&mut self, id: &str) {
        let id = u128::from_str_radix(id, 16).expect("Failed to parse session id");
        self.sessions.retain(|s| s.id != id);
    }

    pub fn add_mfa_count_to_auth_session(&mut self, id: &str) {
        self.get_mut_auth_session_by_id(id).mfa_count += 1;
    }

    pub fn check_password_for_auth_session(&mut self, id: &str, password_hash: &str) -> bool {
        let hash = self.auth.hash.clone();
        let auth_session = self.get_mut_auth_session_by_id(id);
        let user_password_hash = u128_to_32_char_hex_string(auth_session.challenge) + hash.as_str();
        if password_hash == sha256::digest(user_password_hash) {
            auth_session.password_correct = true;
            true
        } else {
            false
        }
    }

    pub fn convert_auth_session_to_session(&mut self, id: &str) -> String {
        let auth_session = self.get_auth_session_by_id(id);
        if auth_session.password_correct {
            if self.auth.mfa.enabled {
                if auth_session.mfa_count >= self.auth.mfa.min_methods {
                    let session_id = self.create_session();
                    self.delete_auth_session_by_id(id);
                    self.save();
                    session_id
                } else {
                    "require_mfas".to_string()
                }
            } else {
                let session_id = self.create_session();
                self.delete_auth_session_by_id(id);
                self.save();
                session_id
            }
        } else {
            "require_password".to_string()
        }
    }

    pub fn delete_all_sessions(&mut self) {
        // delete all sessions and auth sessions
        self.sessions = vec![];
        self.auth.auth_sessions = vec![];
        self.save();
    }

    pub fn delete_all_auth_sessions(&mut self) {
        self.auth.auth_sessions = vec![];
        self.save();
    }
}

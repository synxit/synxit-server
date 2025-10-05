use crate::logger::error::Error;
use crate::synxit::config::CONFIG;
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
            completed_mfa: Vec::new(),
            password_correct: false,
        });
        u128_to_32_char_hex_string(id)
    }

    pub fn get_session_by_id(&self, id: &str) -> Result<&Session, Error> {
        match u128::from_str_radix(id, 16) {
            Ok(id) => self
                .sessions
                .iter()
                .find(|s| s.id == id)
                .ok_or(Error::new("Could not find session")),
            Err(_) => Err(Error::new("Invalid session ID")),
        }
    }

    pub fn get_auth_session_by_id(&self, id: &str) -> Result<&AuthSession, Error> {
        match u128::from_str_radix(id, 16) {
            Ok(id) => self
                .auth
                .auth_sessions
                .iter()
                .find(|s| s.id == id)
                .ok_or(Error::new("Could not find auth session")),
            Err(_) => Err(Error::new("Invalid auth session ID")),
        }
    }

    fn get_mut_auth_session_by_id(&mut self, id: &str) -> Result<&mut AuthSession, Error> {
        match u128::from_str_radix(id, 16) {
            Ok(id) => match self.auth.auth_sessions.iter_mut().find(|s| s.id == id) {
                Some(session) => Ok(session),
                None => Err(Error::new("Could not find auth session")),
            },
            Err(_) => Err(Error::new("Invalid auth session ID")),
        }
    }

    pub fn delete_auth_session_by_id(&mut self, id: &str) {
        if let Some(id) = u128::from_str_radix(id, 16).ok() {
            self.auth.auth_sessions.retain(|s| s.id != id);
        } else {
            log::error!("Invalid auth session ID");
        }
    }

    pub fn delete_session_by_id(&mut self, id: &str) -> bool {
        match u128::from_str_radix(id, 16) {
            Ok(id) => {
                self.sessions.retain(|s| s.id != id);
                true
            }
            Err(_) => false,
        }
    }

    pub fn check_password_for_auth_session(&mut self, id: &str, password_hash: &str) -> bool {
        let hash = self.auth.hash.clone();
        match self.get_mut_auth_session_by_id(id) {
            Ok(auth_session) => {
                let user_password_hash =
                    u128_to_32_char_hex_string(auth_session.challenge) + hash.as_str();
                if password_hash == sha256::digest(user_password_hash) {
                    auth_session.password_correct = true;
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn convert_auth_session_to_session(&mut self, id: &str) -> Result<String, &str> {
        match self.get_auth_session_by_id(id) {
            Ok(auth_session) => {
                if auth_session.password_correct {
                    if self.auth.mfa.enabled {
                        if auth_session.completed_mfa.len() as u8 >= self.auth.mfa.min_methods {
                            let session_id = self.create_session();
                            self.delete_auth_session_by_id(id);
                            self.save();
                            Ok(session_id)
                        } else {
                            Err("require_mfa")
                        }
                    } else {
                        let session_id = self.create_session();
                        self.delete_auth_session_by_id(id);
                        self.save();
                        Ok(session_id)
                    }
                } else {
                    Err("require_password")
                }
            }
            Err(_) => Err("not_found"),
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

    pub fn check_auth_by_id(&self, id: &str) -> bool {
        match self.get_session_by_id(id) {
            Ok(session) => match session
                .last_used
                .checked_add(CONFIG.get().unwrap().auth.session_timeout)
            {
                Some(last_used) => last_used > current_time(),
                None => false,
            },
            Err(_) => false,
        }
    }

    pub fn auth_session_add_completed_mfa(&mut self, id: &str, mfa_id: u8) {
        match self.get_mut_auth_session_by_id(id) {
            Ok(auth_session) => {
                auth_session.completed_mfa.push(mfa_id);
            }
            Err(_) => {}
        }
    }
}

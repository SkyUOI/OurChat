pub mod get_account_info;
pub mod msg_delivery;
pub mod upload;

pub mod v1 {
    include!("../generated/service.ourchat.v1.rs");
}

pub mod download {
    pub mod v1 {
        include!("../generated/service.ourchat.download.v1.rs");
    }
}

pub mod delete {
    pub mod v1 {
        include!("../generated/service.ourchat.delete.v1.rs");
    }
}

pub mod session {
    pub mod new_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.new_session.v1.rs");
        }
    }

    pub mod accept_join_session_invitation {
        pub mod v1 {
            include!("../generated/service.ourchat.session.accept_join_session_invitation.v1.rs");
        }
    }

    pub mod invite_user_to_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.invite_user_to_session.v1.rs");
        }
    }

    pub mod get_session_info {
        pub mod v1 {
            include!("../generated/service.ourchat.session.get_session_info.v1.rs");
        }
    }

    pub mod set_session_info {
        pub mod v1 {
            include!("../generated/service.ourchat.session.set_session_info.v1.rs");
        }
    }

    pub mod set_role {
        pub mod v1 {
            include!("../generated/service.ourchat.session.set_role.v1.rs");
        }
    }

    pub mod add_role {
        pub mod v1 {
            include!("../generated/service.ourchat.session.add_role.v1.rs");
        }
    }

    pub mod get_role {
        pub mod v1 {
            include!("../generated/service.ourchat.session.get_role.v1.rs");
        }
    }

    pub mod mute {
        pub mod v1 {
            include!("../generated/service.ourchat.session.mute.v1.rs");
        }
    }

    pub mod ban {
        pub mod v1 {
            include!("../generated/service.ourchat.session.ban.v1.rs");
        }
    }

    pub mod delete_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.delete_session.v1.rs");
        }
    }

    pub mod leave_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.leave_session.v1.rs");
        }
    }

    pub mod join_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.join_session.v1.rs");
        }
    }

    pub mod allow_user_join_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.allow_user_join_session.v1.rs");
        }
    }

    pub mod session_room_key {
        pub mod v1 {
            include!("../generated/service.ourchat.session.session_room_key.v1.rs");
        }
    }

    pub mod e2eeize_and_dee2eeize_session {
        pub mod v1 {
            include!("../generated/service.ourchat.session.e2eeize_and_dee2eeize_session.v1.rs");
        }
    }

    pub mod kick {
        pub mod v1 {
            include!("../generated/service.ourchat.session.kick.v1.rs");
        }
    }
}

pub mod set_account_info {
    pub mod v1 {
        include!("../generated/service.ourchat.set_account_info.v1.rs");
    }
}

pub mod unregister {
    pub mod v1 {
        include!("../generated/service.ourchat.unregister.v1.rs");
    }
}

pub mod friends {
    pub mod add_friend {
        pub mod v1 {
            include!("../generated/service.ourchat.friends.add_friend.v1.rs");
        }
    }

    pub mod set_friend_info {
        pub mod v1 {
            include!("../generated/service.ourchat.friends.set_friend_info.v1.rs");
        }
    }

    pub mod accept_friend_invitation {
        pub mod v1 {
            include!("../generated/service.ourchat.friends.accept_friend_invitation.v1.rs");
        }
    }

    pub mod delete_friend {
        pub mod v1 {
            include!("../generated/service.ourchat.friends.delete_friend.v1.rs");
        }
    }
}

pub mod webrtc {
    pub mod room {
        pub mod create_room {
            pub mod v1 {
                include!("../generated/service.ourchat.webrtc.room.create_room.v1.rs");
            }
        }
    }
}

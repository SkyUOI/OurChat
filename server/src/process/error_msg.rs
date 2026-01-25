pub mod not_found {
    pub const USER: &str = "User Not Found";
    pub const SESSION: &str = "Session Not Found";
    pub const USER_IN_SESSION: &str = "User Not In Session";
    pub const MSG: &str = "Message Not Found";
    pub const NOT_BE_MUTED: &str = "User Not Be Muted";
    pub const NOT_BE_BANNED: &str = "User Not Be Banned";
    pub const SESSION_INVITATION: &str = "Session Invitation Not Found";
    pub const FRIEND: &str = "Friend Not Found";
    pub const FRIEND_INVITATION: &str = "Friend Invitation Not Found";
    pub const ROLE: &str = "Role Not Found";
    pub const FILE: &str = "File Not Found";
    pub const WEBRTC_ROOM: &str = "WebRTC Room Not Found";
}

pub mod exist {
    pub const USER: &str = "User Already Exists";
    pub const SESSION: &str = "Session Already Exists";
    pub const USER_IN_SESSION: &str = "User Already In Session";
    pub const MSG: &str = "Message Already Exists";
    pub const FRIEND: &str = "Friend Already Exists";
}

pub mod invalid {
    pub const USERNAME: &str = "Username Is Invalid";
    pub const EMAIL_ADDRESS: &str = "Email Address Is Invalid";
    pub const STATUS_TOO_LONG: &str = "Status Too Long";
    pub const OCID_TOO_LONG: &str = "Ocid Too Long";
}

pub const SERVER_ERROR: &str = "Server Error";
pub const PERMISSION_DENIED: &str = "Permission Denied";
pub const REQUEST_INVALID_VALUE: &str = "Request Invalid Value";
pub const NOT_IN_SESSION: &str = "Not In Session";
pub const CONFLICT: &str = "Conflict";
pub const MAINTAINING: &str = "Server Maintaining";
pub const MUTE: &str = "User Muted";
pub const BAN: &str = "User Banned";
pub const ACCOUNT_DELETED: &str = "Account Deleted";
pub const E2EE_NOT_ON: &str = "E2EE Not On";

// fetch msg

pub const TIME_FORMAT_ERROR: &str = "Time Format Error";
pub const TIME_MISSING: &str = "Time Missing";

// upload

pub const FILE_SIZE_ERROR: &str = "File Size Error";
pub const FILE_HASH_ERROR: &str = "File Hash Error";
pub const STORAGE_FULL: &str = "Storage Full";
pub const METADATA_ERROR: &str = "Metadata Error";
pub const INCORRECT_ORDER: &str = "Incorrect Order Of Uploading";

// Set Session Info
pub const CANNOT_SET_NAME: &str = "Cannot Set Name";
pub const CANNOT_SET_DESCRIPTION: &str = "Cannot Set Description";
pub const CANNOT_SET_AVATAR: &str = "Cannot Set Avatar";

// Auth
pub const MISSING_AUTH_TYPE: &str = "Missing AuthType";
pub const WRONG_PASSWORD: &str = "Wrong Password";

// Role
pub const ROLE_NAME_EMPTY: &str = "Role Name Empty";

// Register
pub const NOT_STRONG_PASSWORD: &str = "Password Is Not Strong Enough";

pub mod token {
    pub const INVALID: &str = "Token Invalid";
    pub const EXPIRED: &str = "Token Expired";
    pub const MISSING: &str = "Token Missing";
    pub const UNSUPPORTED_AUTHORIZATION_HEADER: &str = "Only Support Bearer";
    pub const INCORRECT_FORMAT: &str = "Correct format is Authorization: Bearer <jwt>";
}

pub mod webrtc {
    pub const UNSPECIFIED: &str = "Signal Type Must Be Specified";
}

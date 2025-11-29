use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u64)]
pub enum PredefinedRoles {
    Member = 1,
    Admin = 2,
    Owner = 3,
    // Add other roles as needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u64)]
pub enum PredefinedPermissions {
    SendMsg = 1,
    RecallMsg = 2,
    BanUser = 3,
    UnbanUser = 4,
    KickUser = 5,
    SetTitle = 6,
    SetAvatar = 7,
    SetDescription = 8,
    DeleteSession = 9,
    SetRole = 10,
    MuteUser = 11,
    UnmuteUser = 12,
    AcceptJoinRequest = 13,
    E2eeizeAndDee2eeizeSession = 14,
    // Add other permissions as needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum AccountStatus {
    Active = 0,
    Deleted = 1,
    // Add other statuses as needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i64)]
pub enum PredefinedServerManagementPermission {
    PublishAnnouncement = 1,
    BanUser = 2,
    MuteUser = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i64)]
pub enum PredefinedServerManagementRole {
    Admin = 1,
    // Add other server management roles as needed
}

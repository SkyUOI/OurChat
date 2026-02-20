use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Ocid,
    Passwd,
    Name,
    Email,
    Time,
    ResourceUsed,
    FriendLimit,
    FriendsNum,
    PublicUpdateTime,
    UpdateTime,
    Avatar,
    Status,
    AccountStatus,
    DeletedAt,
    PublicKey,
    GithubId,
    OauthProvider,
    EmailVerified,
}

#[derive(DeriveIden)]
pub enum Friend {
    Table,
    UserId,
    #[allow(clippy::enum_variant_names)]
    FriendId,
    SessionId,
}

#[derive(DeriveIden)]
pub enum SessionRelation {
    Table,
    SessionId,
    UserId,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum Session {
    Table,
    #[allow(clippy::enum_variant_names)]
    SessionId,
    Name,
    Size,
    AvatarKey,
    CreatedTime,
    UpdatedTime,
    Description,
    DefaultRole,
    E2EEOn,
    RoomKeyTime,
    LeavingToProcess,
}

#[derive(DeriveIden)]
pub enum UserChatMsg {
    Table,
    ChatMsgId,
    MsgData,
    SenderId,
    SessionId,
    Time,
    IsEncrypted,
    MsgId,
}

#[derive(DeriveIden)]
pub enum Files {
    Table,
    Key,
    Date,
    AutoClean,
    Path,
    UserId,
    SessionId,
}

#[derive(DeriveIden)]
pub enum UserStatus {
    Table,
    Name,
}

#[derive(DeriveIden)]
pub enum UserContactInfo {
    Table,
    UserId,
    ContactUserId,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum ServerManagementPermission {
    Table,
    Id,
    Name,
    Description,
}

#[derive(DeriveIden)]
pub enum ServerManagementRole {
    Table,
    Id,
    Name,
    Description,
}

#[derive(DeriveIden)]
pub enum ServerManagementRolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
pub enum Role {
    Table,
    Id,
    CreatorId,
    Name,
    Description,
    SessionId,
}

#[derive(DeriveIden)]
pub enum Permission {
    Table,
    Id,
    Description,
}

#[derive(DeriveIden)]
pub enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
pub enum UserRoleRelation {
    Table,
    UserId,
    SessionId,
    RoleId,
}

#[derive(DeriveIden)]
pub enum ManagerRoleRelation {
    Table,
    UserId,
    RoleId,
}

#[derive(DeriveIden)]
pub enum Announcement {
    Table,
    Id,
    Title,
    Content,
    CreatedAt,
    PublisherId,
}

#[derive(DeriveIden)]
pub enum AnnouncementMsg {
    Table,
    AnnouncementId,
    MsgId,
}

#[derive(DeriveIden)]
pub enum WebrtcRoom {
    Table,
    Id,
    Title,
    Description,
}

#[derive(DeriveIden)]
pub enum WebrtcRoomMember {
    Table,
    RoomId,
    UserId,
    Name,
}

#[derive(DeriveIden)]
pub enum RTCRoom {
    Table,
    RoomId,
    Title,
    UsersNum,
}

#[derive(DeriveIden)]
pub enum SessionInvitation {
    Table,
    Id,
    Inviter,
    Invitee,
    SessionId,
    LeaveMessage,
    ExpireAt,
}

#[derive(DeriveIden)]
pub enum MessageRecords {
    Table,
    MsgId,
    MsgData,
    SenderId,
    SessionId,
    Time,
    IsEncrypted,
    IsAllUser,
}

#[derive(DeriveIden)]
pub enum MetricsHistory {
    Table,
    Id,
    Timestamp,
    ActiveConnections,
    TotalUsers,
    MessagesPerSecond,
    UptimeSeconds,
    CpuUsagePercent,
    MemoryUsagePercent,
    DiskUsagePercent,
    TotalSessions,
    ActiveSessions,
    DatabaseConnections,
    RabbitmqConnections,
    CreatedAt,
    // Tokio runtime metrics - Stable base metrics
    TokioWorkersCount,
    TokioTotalParkCount,
    TokioMaxParkCount,
    TokioMinParkCount,
    TokioTotalBusyDuration,
    TokioMaxBusyDuration,
    TokioMinBusyDuration,
    TokioGlobalQueueDepth,
    TokioElapsed,
    TokioLiveTasksCount,
    // Tokio runtime metrics - Unstable base metrics
    TokioMeanPollDuration,
    TokioMeanPollDurationWorkerMin,
    TokioMeanPollDurationWorkerMax,
    TokioTotalNoopCount,
    TokioMaxNoopCount,
    TokioMinNoopCount,
    TokioTotalStealCount,
    TokioMaxStealCount,
    TokioMinStealCount,
    TokioTotalStealOperations,
    TokioMaxStealOperations,
    TokioMinStealOperations,
    TokioNumRemoteSchedules,
    TokioTotalLocalScheduleCount,
    TokioMaxLocalScheduleCount,
    TokioMinLocalScheduleCount,
    TokioTotalOverflowCount,
    TokioMaxOverflowCount,
    TokioMinOverflowCount,
    TokioTotalPollsCount,
    TokioMaxPollsCount,
    TokioMinPollsCount,
    TokioTotalLocalQueueDepth,
    TokioMaxLocalQueueDepth,
    TokioMinLocalQueueDepth,
    TokioBlockingQueueDepth,
    TokioBlockingThreadsCount,
    TokioIdleBlockingThreadsCount,
    TokioBudgetForcedYieldCount,
    TokioIoDriverReadyCount,
    // Tokio runtime metrics - Derived metrics
    TokioBusyRatio,
    TokioMeanPollsPerPark,
    // Tokio task metrics - Base metrics
    TokioTaskInstrumentedCount,
    TokioTaskDroppedCount,
    TokioTaskFirstPollCount,
    TokioTaskTotalFirstPollDelay,
    TokioTaskTotalIdledCount,
    TokioTaskTotalIdleDuration,
    TokioTaskMaxIdleDuration,
    TokioTaskTotalScheduledCount,
    TokioTaskTotalScheduledDuration,
    TokioTaskTotalPollCount,
    TokioTaskTotalPollDuration,
    TokioTaskTotalFastPollCount,
    TokioTaskTotalFastPollDuration,
    TokioTaskTotalSlowPollCount,
    TokioTaskTotalSlowPollDuration,
    TokioTaskTotalShortDelayCount,
    TokioTaskTotalShortDelayDuration,
    TokioTaskTotalLongDelayCount,
    TokioTaskTotalLongDelayDuration,
    // Tokio task metrics - Derived metrics
    TokioTaskMeanFirstPollDelay,
    TokioTaskMeanIdleDuration,
    TokioTaskMeanScheduledDuration,
    TokioTaskMeanPollDuration,
    TokioTaskSlowPollRatio,
    TokioTaskLongDelayRatio,
    TokioTaskMeanFastPollDuration,
    TokioTaskMeanSlowPollDuration,
    TokioTaskMeanShortDelayDuration,
    TokioTaskMeanLongDelayDuration,
}

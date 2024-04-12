use shared::object_id::ObjectId;

#[derive(Debug, serde::Deserialize)]
pub struct AuditLog {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub kind: AuditLogKind,
    pub actor_id: ObjectId,
    pub target_id: ObjectId,
    pub target_kind: AuditLogTargetKind,
    // changes: Vec<>,
}

// https://github.com/SevenTV/Common/blob/master/structures/v3/type.audit.go#L21
#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(u32)]
pub enum AuditLogKind {
    CreateEmote = 1,
    DeleteEmote = 2,
    DisableEmote = 3,
    UpdateEmote = 4,
    MergeEmote = 5,
    UndoDeleteEmote = 6,
    EnableEmote = 7,
    ProcessEmote = 8,

    SignUserToken = 20,
    SignCsrfToken = 21,
    RejectedAccess = 26,

    CreateUser = 30,
    DeleteUser = 31,
    BanUser = 32,
    EditUser = 33,
    UnbanUser = 36,

    CreateEmoteSet = 70,
    UpdateEmoteSet = 71,
    DeleteEmoteSet = 72,

    CreateReport = 80,
    UpdateReport = 81,

    ReadMessage = 90,
}

// https://github.com/SevenTV/Common/blob/master/structures/v3/structures.go#L97
#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(u32)]
pub enum AuditLogTargetKind {
    User = 1,
    Emote = 2,
    EmoteSet = 3,
    Role = 4,
    Entitlement = 5,
    Ban = 6,
    Message = 7,
    Report = 8,
    Presence = 9,
    Cosmetic = 10,
}

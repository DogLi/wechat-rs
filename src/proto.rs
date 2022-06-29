use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u32)]
pub enum DataType {
    RecvTxtMsg = 1,
    RecvPicMsg = 3,
    TxtMsg = 555,
    PicMsg = 500,
    AtMsg = 550,
    GetUserListSuccess = 5001,
    GetUserListFail = 5002,
    UserList = 5000,
    HeartBeat = 5005,
    DebugSwitch = 6000,
    PersonalDetail = 6550,
    ChatroomMember = 5010,
    ChatroomMemberNick = 5020,
    PersonalInfo = 6500,
    AttachFile = 5003,
}

#[derive(Clone, Debug, Serialize)]
pub struct DataSend {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: DataType,
    pub roomid: String,
    pub wxid: String,
    pub content: String,
    pub nickname: String,
    pub ext: String,
}

fn get_id() -> String {
    let now = Local::now();
    now.format("%Y%m%d%H%M%S%.6f").to_string()
}

impl Default for DataSend {
    fn default() -> Self {
        Self {
            id: get_id(),
            data_type: DataType::TxtMsg,
            roomid: "null".into(),
            wxid: "null".into(),
            content: "null".into(),
            nickname: "null".into(),
            ext: "null".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response<T> {
    pub content: T,
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: u32,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PersonInfo {
    pub wx_code: String,
    pub wx_id: String,
    pub wx_name: String,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct RoomInfo {
    pub room_id: String,
    #[serde(rename = "member")]
    pub members: Vec<String>,
    pub address: u64,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ContactInfo {
    pub headimg: String,
    pub name: String,
    pub node: u64,
    pub remarks: String,
    pub wxcode: String,
    pub wxid: String,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct RoomMemberInfo {
    pub nick: String,
    pub roomid: String,
    pub wxid: String,
}

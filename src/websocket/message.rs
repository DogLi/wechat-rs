use crate::proto::DataType;
use serde::Deserialize;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize)]
pub enum Message {
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Closed,
    Msg(WechatMsg),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum WechatMsg {
    HeartBeat(HeartBeat),
    TextMsg(TextMsg),
    PicMsg(PicMsg),
}

#[derive(Clone, Debug, Deserialize)]
pub struct HeartBeat {
    pub content: String,
    pub id: String,
    pub receiver: String,
    pub sender: String,
    pub srvid: u64,
    pub status: String,
    pub time: String,
    #[serde(alias = "type")]
    pub data_type: DataType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextMsg {
    pub content: String,
    pub id: String,
    pub id1: String,
    pub id2: String,
    pub id3: String,
    pub srvid: u64,
    pub time: String,
    #[serde(alias = "type")]
    pub data_type: DataType,
    #[serde(alias = "wxid")]
    wx_id: String,
}

impl TextMsg {
    pub fn is_chatroom_msg(&self) -> bool {
        self.wx_id.contains("chatroom")
    }

    pub fn room_id(&self) -> Option<String> {
        if self.wx_id.ends_with("@chatroom") {
            Some(self.wx_id.clone())
        } else {
            None
        }
    }

    pub fn wechat_id(&self) -> String {
        if self.wx_id.contains("chatroom") {
            self.id1.clone()
        } else {
            self.wx_id.clone()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PicDetail {
    pub content: String,
    pub detail: String,
    pub id1: String,
    pub id2: String,
    pub thumb: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PicMsg {
    pub content: PicDetail,
    pub id: String,
    pub receiver: String,
    pub sender: String,
    pub srvid: u64,
    pub status: String,
    pub time: String,
    #[serde(alias = "type")]
    pub data_type: DataType,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserde_wechat_msg() {
        let s = r#"{
            "content":"ee",
            "id":"20220629104242",
            "id1":"",
            "id2":"wxid_pu04n556crxe22",
            "id3":"",
            "srvid":1,
            "time":"2022-06-29 10:42:42",
            "type":1,
            "wxid":"wxid_qf0161up823v22"
            }"#;
        let wechat_msg: Result<WechatMsg, _> = serde_json::from_str(s);
        println!("{:?}", wechat_msg);
        assert!(wechat_msg.is_ok())
    }
}

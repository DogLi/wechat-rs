#[macro_use]
extern crate log;
use anyhow::{bail, Result};
use chrono::Local;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::path::Path;

pub mod data_type {
    pub static HEART_BEAT: u32 = 5005;
    pub static RECV_TXT_MSG: u32 = 1;
    pub static RECV_PIC_MSG: u32 = 3;
    pub static USER_LIST: u32 = 5000;
    pub static GET_USER_LIST_SUCCSESS: u32 = 5001;
    pub static GET_USER_LIST_FAIL: u32 = 5002;
    pub static TXT_MSG: u32 = 555;
    pub static PIC_MSG: u32 = 500;
    pub static AT_MSG: u32 = 550;
    pub static CHATROOM_MEMBER: u32 = 5010;
    pub static CHATROOM_MEMBER_NICK: u32 = 5020;
    pub static PERSONAL_INFO: u32 = 6500;
    pub static DEBUG_SWITCH: u32 = 6000;
    pub static PERSONAL_DETAIL: u32 = 6550;
    pub static ATTATCH_FILE: u32 = 5003;
}

#[derive(Debug, Clone)]
pub struct WechatClient {
    base_url: Url,
}

#[derive(Clone, Debug, Serialize)]
struct DataSend {
    id: String,
    #[serde(rename = "type")]
    data_type: u32,
    // #[serde(skip_serializing_if = "Option::is_none" )]
    roomid: String,
    // #[serde(skip_serializing_if = "Option::is_none" )]
    wxid: String,
    // #[serde(skip_serializing_if = "Option::is_none" )]
    content: String,
    // #[serde(skip_serializing_if = "Option::is_none" )]
    nickname: String,
    // #[serde(skip_serializing_if = "Option::is_none" )]
    ext: String,
}

impl Default for DataSend {
    fn default() -> Self {
        Self {
            id: get_id(),
            data_type: 0,
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

fn get_id() -> String {
    let now = Local::now();
    now.format("%Y%m%d%H%M%S%.6f").to_string()
}

impl WechatClient {
    pub fn new(ip: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}/", ip, port).parse().unwrap();
        Self { base_url }
    }

    async fn send<T: Serialize + Debug, P: DeserializeOwned>(
        &self,
        uri: &str,
        data: T,
    ) -> Result<P> {
        debug!("request: {}", serde_json::to_string(&data).unwrap());
        let url = self.base_url.join(uri)?;
        let client = reqwest::Client::new();
        let request = serde_json::json!({ "para": data });
        let response = client.post(url).json(&request).send().await?;
        let r = response.json().await?;
        debug!("response text: {:?}", r);
        let r: Response<P> = serde_json::from_value(r)?;
        Ok(r.content)
    }

    /// 获取本人用户信息
    pub async fn get_personal_info(&self) -> Result<PersonInfo> {
        let uri = "api/get_personal_info";
        let data = DataSend {
            data_type: data_type::PERSONAL_INFO,
            ..Default::default()
        };
        let s: String = self.send(uri, data).await?;
        let info = serde_json::from_str(&s)?;
        Ok(info)
    }

    /// 获取群信息
    pub async fn get_room_info(&self) -> Result<Vec<RoomInfo>> {
        let uri = "api/getmemberid";
        let data = DataSend {
            data_type: data_type::CHATROOM_MEMBER,
            content: "op:list member".into(),
            ..Default::default()
        };
        self.send(uri, data).await
    }

    /// 获取通讯录信息
    pub async fn get_contact_list(&self) -> Result<Vec<ContactInfo>> {
        let uri = "api/getcontactlist";
        let data = DataSend {
            data_type: data_type::USER_LIST,
            ..Default::default()
        };
        self.send(uri, data).await
    }

    ///  获取指定群的成员的昵称（可用于at）
    pub async fn get_member_nickname(&self, wxid: String, roomid: String) -> Result<String> {
        let uri = "api/getmembernick";
        let data = DataSend {
            data_type: data_type::CHATROOM_MEMBER_NICK,
            wxid,
            roomid,
            ..Default::default()
        };
        let s: String = self.send(uri, data).await?;
        let info: RoomMemberInfo = serde_json::from_str(&s)?;
        Ok(info.nick)
    }

    /// @ 群成员
    pub async fn send_at_msg(
        &self,
        roomid: String,
        wxid: String,
        content: String,
        nickname: String,
    ) -> Result<Value> {
        let uri = "api/sendatmsg";
        let data = DataSend {
            data_type: data_type::AT_MSG,
            roomid,
            content,
            wxid,
            nickname,
            ..Default::default()
        };
        self.send(uri, data).await
    }

    /// 发送图片
    pub async fn send_pic(&self, to_wxid: String, path: &Path) -> Result<Value> {
        let uri = "api/sendpic";
        let data = DataSend {
            data_type: data_type::PIC_MSG,
            wxid: to_wxid,
            content: path.display().to_string(),
            ..Default::default()
        };
        self.send(uri, data).await
    }

    /// 获取所有群的群友
    pub async fn get_chatroom_member_list(&self) -> Result<Vec<RoomInfo>> {
        let uri = "api/get_charroom_member_list";
        let data = DataSend {
            data_type: data_type::CHATROOM_MEMBER,
            ..Default::default()
        };
        self.send(uri, data).await
    }

    /// 发送文字消息
    /// to_id: wx_id 或者 room_id
    pub async fn send_txt_msg(&self, to_id: String, content: String) -> Result<()> {
        let uri = "api/sendtxtmsg";
        let data = DataSend {
            data_type: data_type::TXT_MSG,
            wxid: to_id,
            content,
            ..Default::default()
        };
        let r: String = self.send(uri, data).await?;
        if r.contains("succsessed") {
            Ok(())
        } else {
            bail!(r)
        }
    }

    /// 发送本地文件
    pub async fn send_attach(&self, wxid: String, path: &Path) -> Result<Value> {
        let uri = "api/sendattatch";
        let data = DataSend {
            data_type: data_type::ATTATCH_FILE,
            wxid,
            content: path.display().to_string(),
            ..Default::default()
        };
        self.send(uri, data).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env::var;

    #[test]
    fn test_wechat_serde() {
        let data = DataSend {
            data_type: data_type::CHATROOM_MEMBER,
            ..Default::default()
        };
        let s = serde_json::to_string(&data);
        println!("{:?}", s);
        assert!(s.is_ok());
    }

    #[tokio::test]
    async fn test_wechat() {
        let wx_id = var("WX_ID").unwrap();
        let room_id = var("ROOM_ID").unwrap();
        let client = WechatClient::new(
            &var("WX_IP").unwrap(),
            var("WX_PORT").unwrap().parse().unwrap(),
        );
        let r = client.get_chatroom_member_list().await;
        assert!(r.is_ok());
        let r = client.get_personal_info().await;
        assert!(r.is_ok());
        let r = client.get_room_info().await;
        assert!(r.is_ok());
        let r = client.get_contact_list().await;
        assert!(r.is_ok());
        let r = client
            .get_member_nickname(wx_id.clone(), room_id.clone())
            .await;
        assert!(r.is_ok());
        let r = client
            .send_at_msg(room_id, wx_id.clone(), "hello".into(), "小9菜".into())
            .await;
        assert!(r.is_ok());
        let r = client.get_chatroom_member_list().await;
        assert!(r.is_ok());
        let r = client.send_txt_msg(wx_id, "hello".into()).await;
        println!("{:?}", r);
        assert!(r.is_ok())
    }
}

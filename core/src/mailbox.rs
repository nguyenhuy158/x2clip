//! Hộp thư store-and-forward (ADR-0006). Chỉ là ống vận chuyển byte —
//! mã hoá và ý nghĩa nội dung nằm ở `crypto.rs` / `sync.rs`.

use crate::config::MailboxConfig;
use rusty_s3::actions::{ListObjectsV2, S3Action};
use rusty_s3::{Bucket, Credentials, UrlStyle};
use std::time::Duration;

/// Phân biệt "sai access key" với "mất mạng" — T9 đòi hai thứ này không được
/// lẫn vào nhau, vì cách xử lý khác hẳn: một cái phải báo người dùng sửa
/// config, một cái chỉ cần thử lại.
#[derive(Debug)]
pub enum MailboxError {
    /// 401/403 — key sai, hết hạn, hoặc không có quyền trên bucket này.
    Auth(String),
    /// Không gọi được tới nơi: DNS, timeout, offline.
    Network(String),
    Other(String),
}

impl std::fmt::Display for MailboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth(m) => write!(f, "hộp thư từ chối truy cập (kiểm tra access key): {m}"),
            Self::Network(m) => write!(f, "không kết nối được hộp thư: {m}"),
            Self::Other(m) => write!(f, "lỗi hộp thư: {m}"),
        }
    }
}
impl std::error::Error for MailboxError {}

pub type MailboxResult<T> = Result<T, MailboxError>;

pub trait Mailbox {
    fn put(&self, key: &str, body: &[u8]) -> MailboxResult<()>;
    fn list(&self, prefix: &str) -> MailboxResult<Vec<String>>;
    fn get(&self, key: &str) -> MailboxResult<Vec<u8>>;
    fn delete(&self, key: &str) -> MailboxResult<()>;
}

/// URL ký sẵn sống đủ lâu cho một request. Không lưu lại đâu cả.
const HAN_KY: Duration = Duration::from_secs(300);

pub struct R2Mailbox {
    bucket: Bucket,
    creds: Credentials,
    agent: ureq::Agent,
}

impl R2Mailbox {
    pub fn new(cfg: &MailboxConfig) -> anyhow::Result<Self> {
        let endpoint = cfg.endpoint.parse()?;
        Ok(Self {
            // R2 chỉ nhận path-style.
            bucket: Bucket::new(
                endpoint,
                UrlStyle::Path,
                cfg.bucket.clone(),
                cfg.region.clone(),
            )?,
            creds: Credentials::new(&cfg.access_key_id, &cfg.secret_access_key),
            agent: ureq::Agent::new_with_defaults(),
        })
    }
}

fn phan_loai(e: ureq::Error) -> MailboxError {
    match e {
        ureq::Error::StatusCode(401 | 403) => MailboxError::Auth("HTTP 401/403 từ R2".to_string()),
        ureq::Error::StatusCode(s) => MailboxError::Other(format!("HTTP {s}")),
        other => MailboxError::Network(other.to_string()),
    }
}

impl Mailbox for R2Mailbox {
    fn put(&self, key: &str, body: &[u8]) -> MailboxResult<()> {
        let url = self.bucket.put_object(Some(&self.creds), key).sign(HAN_KY);
        self.agent
            .put(url.as_str())
            .send(body)
            .map_err(phan_loai)
            .map(|_| ())
    }

    fn list(&self, prefix: &str) -> MailboxResult<Vec<String>> {
        let mut action = ListObjectsV2::new(&self.bucket, Some(&self.creds));
        action.with_prefix(prefix.to_string());
        let url = action.sign(HAN_KY);

        let xml = self
            .agent
            .get(url.as_str())
            .call()
            .map_err(phan_loai)?
            .body_mut()
            .read_to_string()
            .map_err(phan_loai)?;

        // ponytail: không phân trang. 1000 key/lần là quá thừa cho hộp thư
        // vốn được dọn sau mỗi lần nhận; thêm continuation token khi nào
        // hộp thư thật sự tồn đọng nghìn item.
        let res = ListObjectsV2::parse_response(&xml)
            .map_err(|e| MailboxError::Other(format!("XML của LIST không đọc được: {e}")))?;
        Ok(res.contents.into_iter().map(|c| c.key).collect())
    }

    fn get(&self, key: &str) -> MailboxResult<Vec<u8>> {
        let url = self.bucket.get_object(Some(&self.creds), key).sign(HAN_KY);
        let mut res = self.agent.get(url.as_str()).call().map_err(phan_loai)?;
        res.body_mut()
            .with_config()
            .limit(crate::MAX_OBJECT_BYTES as u64)
            .read_to_vec()
            .map_err(phan_loai)
    }

    fn delete(&self, key: &str) -> MailboxResult<()> {
        let url = self
            .bucket
            .delete_object(Some(&self.creds), key)
            .sign(HAN_KY);
        self.agent
            .delete(url.as_str())
            .call()
            .map_err(phan_loai)
            .map(|_| ())
    }
}

//! `R2Mailbox` nói chuyện HTTP thật. `FakeMailbox` trong `phase2.rs` cài
//! thẳng trait `Mailbox` nên không đi qua ureq một dòng nào — nghĩa là đường
//! gửi duy nhất chở nội dung chưa từng được kiểm với một response non-2xx.
//!
//! Chỗ này quan trọng vì `push_pending` làm `put(...)?` rồi mới
//! `dat_synced(SYNC_DA_GUI)`. Nếu `put` trả `Ok` cho HTTP 403 thì item bị
//! đánh dấu "đã gửi" trong khi R2 không nhận gì cả — mất item, im lặng, vi
//! phạm N8. Test này chỉ hỏi đúng một câu: non-2xx có thành `Err` không.

use std::io::{Read, Write};
use std::net::TcpListener;
use x2clip_core::config::MailboxConfig;
use x2clip_core::mailbox::{Mailbox, MailboxError, R2Mailbox};

/// Server một-request trả về đúng `status`. Trả về endpoint để trỏ vào.
fn server_tra_ve(status: &'static str) -> String {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = l.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = l.accept() {
            let _ = s.read(&mut [0u8; 4096]);
            let _ =
                s.write_all(format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\n\r\n").as_bytes());
        }
    });
    format!("http://{addr}")
}

fn hop_thu(endpoint: String) -> R2Mailbox {
    R2Mailbox::new(&MailboxConfig {
        endpoint,
        bucket: "x2clip".to_string(),
        region: "auto".to_string(),
        access_key_id: "khoa-test".to_string(),
        secret_access_key: "bi-mat-test".to_string(),
    })
    .expect("dựng được R2Mailbox")
}

#[test]
fn put_403_phai_la_loi_auth() {
    let mb = hop_thu(server_tra_ve("403 Forbidden"));
    match mb.put("inbox/b/01", b"blob") {
        Err(MailboxError::Auth(_)) => {}
        khac => panic!("403 phải ra Auth, đang là: {khac:?}"),
    }
}

#[test]
fn put_500_phai_la_loi_chu_khong_phai_ok() {
    let mb = hop_thu(server_tra_ve("500 Internal Server Error"));
    assert!(
        mb.put("inbox/b/01", b"blob").is_err(),
        "500 mà trả Ok thì push_pending sẽ đánh dấu đã gửi trong khi mất item"
    );
}

#[test]
fn put_khong_ai_nghe_phai_la_loi_mang() {
    // Bind rồi drop: cổng chắc chắn không còn ai nghe.
    let cong = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let mb = hop_thu(format!("http://{cong}"));
    match mb.put("inbox/b/01", b"blob") {
        Err(MailboxError::Network(_)) => {}
        khac => panic!("mất kết nối phải ra Network, đang là: {khac:?}"),
    }
}

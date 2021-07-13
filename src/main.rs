use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
mod s3_utils;
use std::env;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
const TTL: Duration = Duration::from_secs(1); // 1 second
use futures::executor::block_on;

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
    padding: 0,
};

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
    padding: 0,
};

struct HelloFS;

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup called {:?}", name);
        if parent == 1 && name.to_str() == Some("hello.txt") {
            println!("inside if condition");
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else {
            println!("inside else condition");
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr called");
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        println!("read called");
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir called");
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, "hello.txt"),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

struct S3FS;
impl Filesystem for S3FS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup called {:?}", name);
        if parent == 1 && name.to_str() == Some("hello.txt") {
            println!("inside if condition");
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else {
            println!("inside else condition");
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr called");
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        println!("read called");
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }
    #[tokio::main]
    async fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir called");
        let objects = block_on(s3_utils::get_objects("trying-test-bucket")).unwrap();
        println!("{:?}",objects);

        println!("readdir after");
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }
        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        for (i, object) in objects.contents.unwrap_or_default().into_iter().enumerate() {
            let filename = object.key.unwrap();
            println!("index: {} and name: {}", i,filename);
            if reply.add(i as u64, (i + 1) as i64, FileType::RegularFile, filename) {
                break;
            }
        }
        reply.ok();
    }
}


fn main() {
    // env_logger::init();
    println!("starting..");
    let mountpoint = env::args_os().nth(1).unwrap();
    let mut options = vec![
        MountOption::RO,
        MountOption::FSName("trying-test-bucket".to_string()),
    ];
    if let Some(auto_unmount) = env::args_os().nth(2) {
        if auto_unmount.eq("--auto_unmount") {
            options.push(MountOption::AutoUnmount);
        }
    }
    println!("Mountpoint: {:?}", mountpoint);
    println!("options: {:?}", options);
    fuser::mount2(S3FS, mountpoint, &options).unwrap();
}

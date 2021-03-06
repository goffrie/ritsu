use std::io;
use std::thread;
use std::fs::File as StdFile;
use bytes::BytesMut;
use ritsu::executor::Runtime;
use ritsu::action::fs;
use tokio_ritsu::Handle;


#[tokio::main]
async fn main() -> io::Result<()> {
    let tokio_handle = tokio::runtime::Handle::current();
    let (driver, handle) = Handle::new(tokio_handle);

    thread::spawn(move || {
        let mut pool = Runtime::new().unwrap();
        let raw_handle = pool.raw_handle();
        pool.run_until(driver.register(raw_handle))
            .unwrap();
    });

    let fd = StdFile::open("./Cargo.toml")?;
    let stdout = StdFile::create("/dev/stdout")?;
    let mut fd = fs::File::from_std(fd);
    let mut stdout = fs::File::from_std(stdout);

    let fut = async move {
        let mut pos = 0;

        loop {
            let buf = fd.read_at(pos, BytesMut::with_capacity(64)).await?;

            if buf.is_empty() {
                break
            }

            pos += buf.len() as i64;
            stdout.write_at(0, buf.freeze()).await?;
        }

        Ok(()) as io::Result<()>
    };

    handle.spawn(fut).await??;

    Ok(())
}

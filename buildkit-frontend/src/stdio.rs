use std::io::{self, Read, stdin, stdout, Write};
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tokio::io::*;
use tokio::io::unix::AsyncFd;
use tonic::transport::Uri;

#[pin_project]
pub struct StdioSocket {
    #[pin]
    reader: AsyncFd<async_stdio::SourceStdin>,

    #[pin]
    writer: AsyncFd<async_stdio::SourceStdout>,
}

pub async fn stdio_connector(_: Uri) -> io::Result<StdioSocket> {
    StdioSocket::try_new()
}

impl StdioSocket {
    pub fn try_new() -> io::Result<Self> {
        Ok(StdioSocket {
            reader: AsyncFd::new(async_stdio::SourceStdin::try_new(stdin())?)?,
            writer: AsyncFd::new(async_stdio::SourceStdout::try_new(stdout())?)?,
        })
    }
}

impl AsyncRead for StdioSocket {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        let mut poll = self.project();
        let poll = poll.reader.poll_read_ready_mut(cx)?;

        match poll {
            Poll::Ready(mut v) => {
                let buf = buf.initialized_mut();
                v.get_inner_mut().read(buf).unwrap();

                Poll::Ready(Ok(()))
            }
            Poll::Pending => Poll::Pending
        }
        //self.project().reader.poll_read(cx, buf)
    }
}

impl AsyncWrite for StdioSocket {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let mut poll = self.project();
        let poll = poll.writer.poll_write_ready_mut(cx)?;
        match poll {
            Poll::Ready(mut v) => {
                Poll::Ready(Ok(v.get_inner_mut().write(buf).unwrap()))
            }
            Poll::Pending => Poll::Pending
        }
        //self.project().writer.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let mut poll = self.project();
        let poll = poll.writer.poll_write_ready_mut(cx)?;
        match poll {
            Poll::Ready(mut v) => Poll::Ready(v.get_inner_mut().flush()),
            Poll::Pending => Poll::Pending
        }
        //self.project().writer.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
        // let mut poll = self.project();
        // let poll = poll.writer.poll_write_ready_mut(cx)?;
        // match poll {
        //     Poll::Ready(mut v) => Poll::Ready(v.get_inner_mut().flush()),
        //     Poll::Pending => Poll::Pending
        // }
        //self.project().writer.poll_shutdown(cx)
    }
}

mod async_stdio {
    use std::io::{self, Read, Stdin, Stdout, Write};
    use std::os::fd::RawFd;
    use std::os::unix::io::AsRawFd;

    use libc::{F_GETFL, F_SETFL, fcntl, O_NONBLOCK};
    use mio::{Interest, Registry, Token};
    use mio::event::Source;
    use mio::unix::SourceFd;

    pub struct SourceStdin(Stdin);
    pub struct SourceStdout(Stdout);

    impl SourceStdin {
        pub fn try_new(stdin: Stdin) -> io::Result<Self> {
            set_non_blocking_flag(&stdin)?;

            Ok(SourceStdin(stdin))
        }
    }

    impl AsRawFd for SourceStdin {
        fn as_raw_fd(&self) -> RawFd {
            return self.0.as_raw_fd()
        }
    }

    impl SourceStdout {
        pub fn try_new(stdout: Stdout) -> io::Result<Self> {
            set_non_blocking_flag(&stdout)?;

            Ok(SourceStdout(stdout))
        }
    }

    impl AsRawFd for SourceStdout {
        fn as_raw_fd(&self) -> RawFd {
            return self.0.as_raw_fd()
        }
    }

    impl Source for SourceStdin {
        fn register(&mut self, registry: &Registry, token: Token, interests: Interest) -> io::Result<()> {
            SourceFd(&self.0.as_raw_fd()).register(registry, token, interests)
        }

        fn reregister(&mut self, registry: &Registry, token: Token, interests: Interest) -> io::Result<()> {
            SourceFd(&self.0.as_raw_fd()).reregister(registry, token, interests)
        }

        fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
            SourceFd(&self.0.as_raw_fd()).deregister(registry)
        }
    }

    impl Read for SourceStdin {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Source for SourceStdout {
        fn register(&mut self, registry: &Registry, token: Token, interests: Interest) -> io::Result<()> {
            SourceFd(&self.0.as_raw_fd()).register(registry, token, interests)
        }

        fn reregister(&mut self, registry: &Registry, token: Token, interests: Interest) -> io::Result<()> {
            SourceFd(&self.0.as_raw_fd()).reregister(registry, token, interests)
        }

        fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
            SourceFd(&self.0.as_raw_fd()).deregister(registry)
        }
    }

    impl Write for SourceStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    fn set_non_blocking_flag<T: AsRawFd>(stream: &T) -> io::Result<()> {
        let flags = unsafe { fcntl(stream.as_raw_fd(), F_GETFL, 0) };

        if flags < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if unsafe { fcntl(stream.as_raw_fd(), F_SETFL, flags | O_NONBLOCK) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}

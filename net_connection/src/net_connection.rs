use failure::Error;

pub type NetResult<T> = Result<T, Error>;
pub type JsonString = String;

pub type NetHandler = Box<FnMut(NetResult<JsonString>) -> NetResult<()> + Send>;

pub trait NetConnection {
    fn send(&mut self, data: JsonString) -> NetResult<()>;
}

pub trait NetWorker {
    fn destroy(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    fn receive(&mut self, _data: JsonString) -> NetResult<()> {
        Ok(())
    }

    fn tick(&mut self) -> NetResult<bool> {
        Ok(false)
    }
}

pub trait NetWorkerFactory: Send {
    fn new(&self, handler: NetHandler) -> NetResult<Box<NetWorker>>;
}

pub struct NetConnectionRelay {
    worker: Box<NetWorker>,
}

impl NetConnection for NetConnectionRelay {
    fn send(&mut self, data: JsonString) -> NetResult<()> {
        self.worker.receive(data)?;
        Ok(())
    }
}

impl NetConnectionRelay {
    pub fn destroy(self) -> NetResult<()> {
        self.worker.destroy()?;
        Ok(())
    }

    pub fn tick(&mut self) -> NetResult<bool> {
        self.worker.tick()
    }

    pub fn new(handler: NetHandler, worker_factory: Box<NetWorkerFactory>) -> NetResult<Self> {
        Ok(NetConnectionRelay {
            worker: worker_factory.new(handler)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;

    struct DefWorker;

    impl NetWorker for DefWorker {}

    struct DefWorkerFactory;

    impl NetWorkerFactory for DefWorkerFactory {
        fn new(&self, _handler: NetHandler) -> NetResult<Box<NetWorker>> {
            Ok(Box::new(DefWorker))
        }
    }

    #[test]
    fn it_can_defaults() {
        let factory = DefWorkerFactory;
        let mut con =
            NetConnectionRelay::new(Box::new(move |_r| Ok(())), Box::new(factory)).unwrap();

        con.send("test".into()).unwrap();
        con.tick().unwrap();
        con.destroy().unwrap();
    }

    struct Worker {
        handler: NetHandler,
    }

    impl NetWorker for Worker {
        fn tick(&mut self) -> NetResult<bool> {
            (self.handler)(Ok("tick".into()))?;
            Ok(true)
        }

        fn receive(&mut self, data: JsonString) -> NetResult<()> {
            (self.handler)(Ok(data))
        }
    }

    struct WorkerFactory;

    impl NetWorkerFactory for WorkerFactory {
        fn new(&self, handler: NetHandler) -> NetResult<Box<NetWorker>> {
            Ok(Box::new(Worker { handler }))
        }
    }

    #[test]
    fn it_invokes_connection_relay() {
        let (sender, receiver) = mpsc::channel();

        let factory = WorkerFactory;
        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(factory),
        ).unwrap();

        con.send("test".into()).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("test".to_string(), res);

        con.destroy().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = mpsc::channel();

        let factory = WorkerFactory;
        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(factory),
        ).unwrap();

        con.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), res);

        con.destroy().unwrap();
    }
}

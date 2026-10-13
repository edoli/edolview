use std::sync::mpsc;

pub struct NotifierSender<T> {
    data_tx: mpsc::Sender<T>,
    notify_tx: mpsc::Sender<()>,
}

impl<T> NotifierSender<T> {
    pub fn send(&self, value: T) -> Result<(), mpsc::SendError<T>> {
        self.data_tx.send(value)?;
        let _ = self.notify_tx.send(());
        Ok(())
    }
}

impl<T> Clone for NotifierSender<T> {
    fn clone(&self) -> Self {
        Self {
            data_tx: self.data_tx.clone(),
            notify_tx: self.notify_tx.clone(),
        }
    }
}

pub fn mpsc_with_notify<T>() -> (NotifierSender<T>, mpsc::Receiver<T>, mpsc::Receiver<()>) {
    let (data_tx, data_rx) = mpsc::channel::<T>();
    let (notify_tx, notify_rx) = mpsc::channel::<()>();

    let tx = NotifierSender { data_tx, notify_tx };

    (tx, data_rx, notify_rx)
}

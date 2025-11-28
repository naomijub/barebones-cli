pub fn to_crossbeam_receiver<T: Send + 'static>(
    std_rx: std::sync::mpsc::Receiver<T>,
) -> crossbeam_channel::Receiver<T> {
    let (cb_tx, cb_rx) = crossbeam_channel::unbounded();

    std::thread::spawn(move || {
        for msg in std_rx {
            // Forward each message
            if cb_tx.send(msg).is_err() {
                break; // crossbeam receiver dropped
            }
        }
    });

    cb_rx
}

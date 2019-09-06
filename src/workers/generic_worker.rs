use std::thread::JoinHandle;
use std::thread;

pub fn run_thread<T: Send + 'static, F: Fn(T) + Send + 'static>(worker : F, params : T) -> JoinHandle<()> {
	thread::spawn(move|| worker (params))
}
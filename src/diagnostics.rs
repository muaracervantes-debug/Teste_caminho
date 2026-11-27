use std::{
    any::Any,
    backtrace::Backtrace,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn log_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("teste_caminho.log")))
        .unwrap_or_else(|| PathBuf::from("teste_caminho.log"))
}

pub fn record(message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    let line = format!("[{timestamp}] {message}\n");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = file.write_all(line.as_bytes());
    }

    eprintln!("{}", message);
}

pub fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else {
        "panic sem mensagem textual".to_owned()
    }
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|location| format!("{}:{}:{}", location.file(), location.line(), location.column()))
            .unwrap_or_else(|| "local desconhecido".to_owned());

        let message = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&'static str>().copied())
            .unwrap_or("panic sem mensagem textual");

        record(&format!(
            "PANIC em {location}: {message}\nBacktrace:\n{}",
            Backtrace::force_capture()
        ));
    }));
}

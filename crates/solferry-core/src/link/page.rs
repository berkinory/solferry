use std::fmt::Write;

use super::files::Served;

const PAGE: &str = include_str!("page.html");

pub(super) fn share(files: &[Served]) -> String {
    let mut rows = String::new();
    for (index, file) in files.iter().enumerate() {
        let _ = write!(
            rows,
            r#"<div class="row"><span class="name">{}</span><span class="size">{}</span><a class="get" href="f/{index}" download>Download</a></div>"#,
            escape(&file.name),
            size(file.size)
        );
    }
    let all = if files.len() > 1 {
        r#"<button class="wide" onclick="document.querySelectorAll('a.get').forEach((link, i) => setTimeout(() => link.click(), i * 600))">Download all</button>"#
    } else {
        ""
    };
    let count =
        if files.len() == 1 { "1 file".to_owned() } else { format!("{} files", files.len()) };
    fill(&format!(
        r#"<h1>Files for you</h1><p>{count} shared from a device on this network.</p><div class="card">{rows}</div>{all}"#
    ))
}

pub(super) fn receive() -> String {
    fill(
        r#"<h1>Send files</h1><p>They go straight to the device that showed you this page.</p>
<label id="drop"><input id="pick" type="file" multiple>Drop files here or tap to choose</label>
<div id="sent" class="card"></div>
<script>
const drop = document.getElementById('drop'), list = document.getElementById('sent');
let queue = Promise.resolve();
function send(files) {
  for (const file of files) {
    const row = document.createElement('div'); row.className = 'row';
    const name = document.createElement('span'); name.className = 'name'; name.textContent = file.name;
    const state = document.createElement('span'); state.className = 'state'; state.textContent = 'Waiting';
    row.append(name, state); list.append(row);
    queue = queue.then(() => new Promise(done => {
      const request = new XMLHttpRequest();
      request.open('PUT', 'u/' + encodeURIComponent(file.name));
      request.upload.onprogress = e => { if (e.lengthComputable) state.textContent = Math.floor(e.loaded / e.total * 100) + '%'; };
      request.onload = () => { const ok = request.status === 201; state.textContent = ok ? 'Sent' : 'Failed'; state.classList.toggle('failed', !ok); done(); };
      request.onerror = () => { state.textContent = 'Failed'; state.classList.add('failed'); done(); };
      request.send(file);
    }));
  }
}
document.getElementById('pick').onchange = e => { send(e.target.files); e.target.value = ''; };
drop.ondragover = e => { e.preventDefault(); drop.classList.add('over'); };
drop.ondragleave = () => drop.classList.remove('over');
drop.ondrop = e => { e.preventDefault(); drop.classList.remove('over'); send(e.dataTransfer.files); };
</script>"#,
    )
}

fn fill(content: &str) -> String {
    PAGE.replace("@CONTENT@", content)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let (mut unit, mut scale) = (0, 1u64);
    while unit < UNITS.len() - 1 && bytes / scale >= 1000 {
        scale *= 1000;
        unit += 1;
    }
    if unit == 0 {
        return format!("{bytes} B");
    }
    let tenths = bytes.saturating_mul(10) / scale;
    format!("{}.{} {}", tenths / 10, tenths % 10, UNITS[unit])
}

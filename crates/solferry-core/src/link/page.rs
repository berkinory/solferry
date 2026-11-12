use std::fmt::Write;

use super::files::Served;

const PAGE: &str = include_str!("page.html");
pub(super) const FONT_REGULAR: &[u8] =
    include_bytes!("../../../solferry-desktop/assets/fonts/InterTight-Regular.ttf");
pub(super) const FONT_MEDIUM: &[u8] =
    include_bytes!("../../../solferry-desktop/assets/fonts/InterTight-Medium.ttf");

pub(super) fn share(files: &[Served]) -> String {
    let mut rows = String::new();
    for (index, file) in files.iter().enumerate() {
        let _ = write!(
            rows,
            r##"<div class="row"><svg class="i"><use href="#file"/></svg><span class="name">{}</span><span class="size">{}</span><a class="get" href="f/{index}" download aria-label="Download"><svg class="i"><use href="#download"/></svg></a></div>"##,
            escape(&file.name),
            size(file.size)
        );
    }
    let all = if files.len() > 1 {
        r##"<button class="primary" onclick="document.querySelectorAll('a.get').forEach((link, i) => setTimeout(() => link.click(), i * 600))"><svg class="i"><use href="#download"/></svg>Download all</button>"##
    } else {
        ""
    };
    let count =
        if files.len() == 1 { "1 file".to_owned() } else { format!("{} files", files.len()) };
    fill(&format!(
        r#"<h1>Files for you</h1><p>{count} shared from a device on this network.</p><div class="frame"><div class="well">{rows}</div></div>{all}"#
    ))
}

pub(super) fn receive() -> String {
    fill(
        r##"<h1>Send files or text</h1><p>They go straight to the device that showed you this page.</p>
<div class="frame"><div class="well">
<label id="drop"><input id="pick" type="file" multiple><span class="badge"><svg class="i"><use href="#upload"/></svg></span>Drop files here or tap to choose</label>
<div id="sent"></div>
</div></div>
<div class="frame text"><div class="well">
<textarea id="text" rows="4" placeholder="Type or paste text"></textarea>
<div class="send"><span id="text-state" class="state"></span><button id="send-text" class="primary" disabled>Send text</button></div>
</div></div>
<script>
const drop = document.getElementById('drop'), list = document.getElementById('sent');
let queue = Promise.resolve();
function send(files) {
  for (const file of files) {
    const row = document.createElement('div'); row.className = 'row';
    row.innerHTML = '<svg class="i"><use href="#file"/></svg><div class="file"><span class="name"></span><div class="bar"><i></i></div></div><span class="state">Waiting</span>';
    row.querySelector('.name').textContent = file.name;
    const state = row.querySelector('.state'), bar = row.querySelector('.bar i');
    const finish = ok => { state.textContent = ok ? 'Sent' : 'Failed'; state.className = 'state ' + (ok ? 'sent' : 'failed'); bar.parentNode.remove(); };
    list.append(row);
    queue = queue.then(() => new Promise(done => {
      const request = new XMLHttpRequest();
      request.open('PUT', 'u/' + encodeURIComponent(file.name));
      request.upload.onprogress = e => {
        if (!e.lengthComputable) return;
        const share = Math.floor(e.loaded / e.total * 100);
        state.textContent = share + '%'; bar.style.width = share + '%';
      };
      request.onload = () => { finish(request.status === 201); done(); };
      request.onerror = () => { finish(false); done(); };
      request.send(file);
    }));
  }
}
const text = document.getElementById('text'), sendText = document.getElementById('send-text'), textState = document.getElementById('text-state');
text.oninput = () => { sendText.disabled = !text.value.trim(); textState.textContent = ''; };
sendText.onclick = async () => {
  sendText.disabled = true;
  const ok = await fetch('t', { method: 'PUT', body: text.value }).then(r => r.status === 201, () => false);
  textState.textContent = ok ? 'Sent' : 'Failed'; textState.className = 'state ' + (ok ? 'sent' : 'failed');
  if (ok) text.value = ''; else sendText.disabled = false;
};
document.getElementById('pick').onchange = e => { send(e.target.files); e.target.value = ''; };
drop.ondragover = e => { e.preventDefault(); drop.classList.add('over'); };
drop.ondragleave = () => drop.classList.remove('over');
drop.ondrop = e => { e.preventDefault(); drop.classList.remove('over'); send(e.dataTransfer.files); };
</script>"##,
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

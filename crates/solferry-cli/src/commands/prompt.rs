use std::io::{BufRead, Write};
use std::path::PathBuf;

use indicatif::HumanBytes;
use iroh::EndpointId;
use solferry_core::{IncomingOffer, PairingRequest};
use tokio::sync::{mpsc, oneshot};

pub enum Question {
    Offer(IncomingOffer, PathBuf),
    Pairing(PairingRequest),
}

enum Answer {
    Accept,
    Decline,
    Block,
}

/// Asks questions one at a time, so each answer line on stdin belongs to the oldest open question.
/// Devices to block go back through `blocks`, since only the receive loop holds the node.
pub fn spawn(
    json: bool,
    blocks: mpsc::UnboundedSender<EndpointId>,
) -> mpsc::UnboundedSender<Question> {
    let (questions, mut queue) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(question) = queue.recv().await {
            if let Some(peer) = ask(question, json).await {
                let _ = blocks.send(peer);
            }
        }
    });
    questions
}

/// Answers one question and returns the peer to block, if that was the answer.
async fn ask(question: Question, json: bool) -> Option<EndpointId> {
    match question {
        Question::Offer(offer, destination) => {
            // The sender gave up or the wait ran out while earlier questions were open.
            if offer.is_closed() {
                return None;
            }
            let peer = offer.summary.peer;
            let prompt = if json {
                super::json::offer(&offer.summary).to_string()
            } else {
                offer_prompt(&offer)
            };
            match read_answer(prompt, json).await {
                Answer::Accept => offer.accept(destination),
                Answer::Decline => offer.decline(),
                Answer::Block => {
                    offer.decline();
                    return Some(peer);
                }
            }
        }
        Question::Pairing(request) => {
            if request.is_closed() {
                return None;
            }
            let peer = request.peer;
            let prompt = if json {
                super::json::pairing_request(&request).to_string()
            } else {
                format!(
                    "{} wants to pair. Code: {}. Is it the same on both screens? [y/N/block] ",
                    request.peer_name, request.code
                )
            };
            match read_answer(prompt, json).await {
                Answer::Accept => request.accept(),
                Answer::Decline => request.decline(),
                Answer::Block => {
                    request.decline();
                    return Some(peer);
                }
            }
        }
    }
    None
}

fn offer_prompt(offer: &IncomingOffer) -> String {
    let summary = &offer.summary;
    let trust = if summary.paired { "paired" } else { "NOT PAIRED" };
    format!(
        "{} ({trust}) wants to send {}, {}, starting with {}. Accept? [y/N/block] ",
        summary.peer_name,
        super::receive::file_count(summary.files),
        HumanBytes(summary.bytes),
        summary.first_path,
    )
}

/// Reads on a plain thread: a blocking read on a runtime thread would keep the process alive
/// after the receive loop ends.
async fn read_answer(prompt: String, json: bool) -> Answer {
    if json {
        println!("{prompt}");
    } else {
        print!("{prompt}");
        let _ = std::io::stdout().flush();
    }
    let (reply, line) = oneshot::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let read = std::io::stdin().lock().read_line(&mut line);
        let _ = reply.send(matches!(read, Ok(n) if n > 0).then_some(line));
    });
    match line.await.ok().flatten().as_deref().map(|line| line.trim().to_lowercase()) {
        Some(answer) if matches!(answer.as_str(), "accept" | "a" | "y" | "yes") => Answer::Accept,
        Some(answer) if matches!(answer.as_str(), "block" | "b") => Answer::Block,
        _ => Answer::Decline,
    }
}

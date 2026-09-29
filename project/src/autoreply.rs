//imports

//scan to login 

//message recieving and assigning to variable

//add dynamic user lists to keep tracked of already replied keywords

//add a gate for trigger keys to prevent duplicate replies

//make a bunch of lists/dictionaries to add replies

//trigger all lists replies or else go to dynamic replies

//make dynamic replies with functions

//impliment rate input on lunch

//add gui indicator for being live and exit button







//imports
use tokio;
// (imagine core async and web socket crates here)

//scan to login 
fn handle_qr_auth() {
    println!("Scan the QR code to connect to WhatsApp session...");
}

//message recieving and assigning to variable
struct IncomingMessage {
    sender: String,
    text: String,
}

//make a bunch of lists/dictionaries to add replies
struct StaticReplies {
    keywords: std::collections::HashMap<String, String>,
}

//trigger all lists replies or else go to dynamic replies
fn process_reply(msg: &IncomingMessage, static_replies: &StaticReplies) -> String {
    if let Some(reply) = static_replies.keywords.get(&msg.text) {
        reply.clone()
    } else {
        generate_dynamic_reply(&msg.text)
    }
}

//make dynamic replies with functions
fn generate_dynamic_reply(input: &str) -> String {
    format!("AI/Dynamic response processing for: '{}'", input)
}

//impliment rate input on lunch
fn check_rate_limit(user_id: &str) -> bool {
    // track timestamps to avoid spamming
    true 
}

//add gui indicator for being live and exit button
// (placeholder for a lightweight native UI framework or status flag)
//! # bmOS_client
//!
//! bmOS_client is an executable in charge of receiving and parsing JSON input from stdin and sending intents with enough confidence (>0.6) to the address and port provided (to bmOS_server)
//!
//! ## Expected input
//! bmOS_client expects JSON strings with the format specified [here](http://voice2json.org/formats.html), and looks for the intent's name and confidence, sending it if the confidence's enough.
//!
//! ## Warnings
//! - bmOS_client is dumb. It will not check whether the intent it's sending is correctly defined in bmOS_server, so sending undefined intents will result in a panic in the target. 
//! - bmOS_client will panic if the server is unavailable upon sending an intent, or if the input it receives does not conform to the JSON input it expects. There are no attempts on recovering lost connections.
//! - bmOS_server needs to be running and listening for new connections before bmOS_client starts.
//!
//! ## Recommended setup
//! Everything is designed around voice2json as the source of intent recognition. The documentation is available [here](http://voice2json.org/#getting-started). The best results have been obtained with the [default profiles](http://voice2json.org/#supported-languages), both in English and Spanish, though one should expect a moderate amount of false positives if the environment is noisy.
//! Below is an example of a barebones sentences.ini file (which is the only modification I did on the downloaded profiles):
//! ```toml
//! [hello]
//! hi beemo
//! hello beemo
//!
//! [song]
//! play a song beemo
//!
//! [sad]
//! you are ugly beemo
//!
//! [angry]
//! i hate you beemo
//!
//! [surprise]
//! surprise beemo
//!
//! [chronometer]
//! start a chronometer beemo
//! give me a chronometer beemo
//!
//! [5more]
//! give it (five | 5) minutes more
//!
//! [10more]
//! give it (ten | 10) minutes more
//!
//! [20more]
//! give it (twenty | 20) minutes more
//!
//! [5less]
//! take (five | 5) minutes less
//!
//! [10less]
//! take (ten | 10) minutes less
//!
//! [20less]
//! take (twenty | 20) minutes less
//!
//! [done]
//! it is done beemo
//! i have finished beemo
//!```
//! 
//! Below is an example of a bash script which sets up audio streaming from bmOS_server's host, receives it on the local host running bmOS_client and pipes it through voice2json up to stdout (which should then be piped to bmOS_client)
//! ```bash 
//! ssh pi@[ip_address_here] "rec -c 2 -t wav -" | sox - -d -t raw -r 22.05k -b 8 - gain -5 | sudo ./voice2json.bash --profile /profile/ transcribe-stream --audio-source - | sudo ./voice2json.bash --profile /profile/ recognize-intent
//! ```
//!
//! Where voice2json.bash contains a script to fire up voice2json's docker container, where en-us_kaldi-zamia is the name of the profile used, and should be changed to whichever is in use:
//! ```bash
//! docker run -i \
//!      --init \
//!      -v "[path_to_local_dir]/voice2json/profile:/profile/" \
//!      -v "[path_to_local_dir]/voice2json/profile:/root/.local/share/voice2json/en-us_kaldi-zamia/" \
//!      -w "$(pwd)" \
//!      -e "HOME=${HOME}" \
//!      synesthesiam/voice2json "$@"
//! ```

use json_minimal::*;


/// Returns a tuple consisting of the intent's name and its confidence,
/// extracted from the provided json line.
///
/// # Panic
/// The function will panic if the json string is incorrectly formatted.
///
/// # Example input
/// Directly from voice2json:
///
/// {"text": "hi beemo", "likelihood": 1.0, "transcribe_seconds": 5.955755680000948, "wav_seconds": 7.136, "tokens": ["hi", "beemo"], "timeout": false, "intent": {"name": "hi_bmo", "confidence": 1.0}, "entities": [], "raw_text": "hi beemo", "recognize_seconds": 0.00013091099935991224, "raw_tokens": ["hi", "beemo"], "speech_confidence": null, "wav_name": null, "slots": {}}
pub fn parse_intent(line: &[u8]) -> (String, f64) {
    let json = match Json::parse(line) {
        Ok(json) => {
            json
        },
        Err( (position,message) ) => {
            panic!("Error on {} at position {}!!!",message, position);
        }
    };

    let search = "intent";

    let intent_json =   match json.get(search) {
                            Some(a) => a,
                            None => panic!("intent not found in the provided json string"),
                        };

    let intent_name = 
        match intent_json.unbox() {
            Json::OBJECT {name: _, value} =>
                match value.unbox() {
                    Json::JSON(values) => {
                        assert_eq!(values.len(),2);

                        match &values[0] {
                            Json::OBJECT {name: _, value} => {
                                value.unbox()
                            },
                            json => {
                                panic!("Couldn't parse the intent's name, found {:?}!!!",json);
                            }
                        }
                    },
            json => {
                panic!("Couldn't parse the intent's name, found {:?}!!!",json)
            }
                }

            _ => panic!("Couldn't parse the intent's name, found {:?}!!!",json)
        };

    let intent_confidence = 
        match intent_json.unbox() {
            Json::OBJECT {name: _, value} =>
                match value.unbox() {
                    Json::JSON(values) => {
                        // We already know its length is 2

                        match &values[1] {
                            Json::OBJECT { name: _ , value } => {
                                value.unbox()
                            },
                            json => {
                                panic!("Couldn't parse the intent's confidence, found {:?}!!!",json);
                            }
                        }
                    },
            json => {
                panic!("Couldn't parse the intent's confidence, found {:?}!!!",json)
            }
                }

            _ => panic!("Couldn't parse the intent's confidence, found {:?}!!!",json)
        };

    let intent_name = match intent_name {
        Json::STRING(val) => val,
        _ => panic!("The intent's name wasn't a string")
    };

    let intent_confidence = match intent_confidence {
        Json::NUMBER(val) => val,
        _ => panic!("The intent's confidence wasn't a number")
    };

    (intent_name.to_string(), *intent_confidence) // return a tuple with the results 
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_message_passing() {
        use std::io::prelude::*;
        use std::net::{TcpListener, TcpStream};
        use std::{thread,time};

        let thread = thread::spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:50000").unwrap();
            for stream in listener.incoming().take(1) {
                //let mut buffer = [0; 1024];
                let mut cosa : String = "".to_string();

                stream.unwrap()
                    //.read(&mut buffer).unwrap();
                    .read_to_string(&mut cosa).unwrap();

                println!("Got: {}", cosa);
                assert_eq!(cosa, "hi_bmo,2.5".to_string());
            }
        });

        thread::sleep(time::Duration::from_secs(3));

        let mut stream = TcpStream::connect("127.0.0.1:50000").unwrap();

        let (name, conf) = ("hi_bmo", 2.5);

        stream.write(format!("[{},{}]", name, conf).as_bytes()).unwrap();

        println!("wrote to stream");

        thread.join().unwrap();
    }
}

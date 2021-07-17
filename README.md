 # bmOS_client

 bmOS_client is an executable in charge of receiving and parsing JSON input from stdin and sending intents with enough confidence (>0.6) to the address and port provided (to bmOS_server). This is part of the software required to run my custom BMO-Boy. Images and a blog post are coming soon.
 More in-depth documentation is available [here](https://docs.rs/bmos_client/).
 
 The documentation and setup information for bmOS_server, the other software component which powers BMO itself, is also available [here](https://docs.rs/crate/bmos_server/), with the source located [here](https://github.com/Sondeluz/bmOS_server)
 

 ## Expected input
 bmOS_client expects JSON strings with the format specified [here](http://voice2json.org/formats.html), and looks for the intent's name and confidence, sending it if the confidence's enough.

 ## Warnings
 - bmOS_client is dumb. It will not check whether the intent it's sending is correctly defined in bmOS_server, so sending undefined intents will result in a panic in the target. 
 - bmOS_client will panic if the server is unavailable upon sending an intent, or if the input it receives does not conform to the JSON input it expects. There are no attempts on recovering lost connections.
 - bmOS_server needs to be running and listening for new connections before bmOS_client starts.

 ## Recommended setup
 Everything is designed around voice2json as the source of intent recognition. The documentation is available [here](http://voice2json.org/#getting-started). The best results have been obtained with the [default profiles](http://voice2json.org/#supported-languages), both in English and Spanish, though one should expect a moderate amount of false positives if the environment is noisy.
 Below is an example of a barebones sentences.ini file (which is the only modification I did on the downloaded profiles):
 ```
 [hello]
 hi beemo
 hello beemo

 [song]
 play a song beemo

 [sad]
 you are ugly beemo

 [angry]
 i hate you beemo

 [surprise]
 surprise beemo

 [chronometer]
 start a chronometer beemo
 give me a chronometer beemo

 [5more]
 give it (five | 5) minutes more

 [10more]
 give it (ten | 10) minutes more

 [20more]
 give it (twenty | 20) minutes more

 [5less]
 take (five | 5) minutes less

 [10less]
 take (ten | 10) minutes less

 [20less]
 take (twenty | 20) minutes less

 [done]
 it is done beemo
 i have finished beemo
```
 
 Below is an example of a bash script which sets up audio streaming from bmOS_server's host, receives it on the local host running bmOS_client and pipes it through voice2json up to stdout (which should then be piped to bmOS_client)
 ```bash 
 ssh pi@[ip_address_here] "rec -c 2 -t wav -" | sox - -d -t raw -r 22.05k -b 8 - gain -5 | sudo ./voice2json.bash --profile /profile/ transcribe-stream --audio-source - | sudo ./voice2json.bash --profile /profile/ recognize-intent
 ```

 Where voice2json.bash contains a script to fire up voice2json's docker container, where en-us_kaldi-zamia is the name of the profile used, and should be changed to whichever is in use:
 ```bash
 docker run -i \
      --init \
      -v "[path_to_local_dir]/voice2json/profile:/profile/" \
      -v "[path_to_local_dir]/voice2json/profile:/root/.local/share/voice2json/en-us_kaldi-zamia/" \
      -w "$(pwd)" \
      -e "HOME=${HOME}" \
      s

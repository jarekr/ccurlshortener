# ccurlshortener - a url shortening http service


This is a subbmission for https://codingchallenges.fyi/challenges/challenge-url-shortener

## Building

 cargo build

## Running

runs on http://localhost:8000

 cargo run

 curl -f -X POST localhost:8000/shorten -d "https://google.com"
 curl -f -X POST localhost:8000/shorten -d "https://lichess.org"
 curl -f -X GET "http://localhost:8000/e/35334e75d4f2a253"
 curl -f -X GET "http://localhost:8000/e/77d4885785192f11"
 

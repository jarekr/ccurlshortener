# ccurlshortener - a url shortening http service

This is a subbmission for https://codingchallenges.fyi/challenges/challenge-url-shortener

Still a work in progress, e.g. no ability to delete shortened URLs. 

## Building

```
cargo build
```

## Running

```
cargo run
```

Go to http://localhost:8000 to see the web ui. Can also hit api endpoints like
so:

# add shortened urls for some websites
curl -i -X POST localhost:8000/shorten -d "https://google.com"
curl -i -X POST localhost:8000/shorten -d "https://lichess.org"

# fetch some known shortened urls
curl -i -X GET "http://localhost:8000/e/35334e75d4f2a253"
curl -i -X GET "http://localhost:8000/e/77d4885785192f11"


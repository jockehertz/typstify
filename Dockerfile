FROM alpine:latest

COPY /target/release/typstify ./typstify

CMD ./typstify

FROM rust:1.31

WORKDIR /app
COPY ./ ./

RUN cargo install --path . --locked

EXPOSE 8000
CMD ["aws_sts"]

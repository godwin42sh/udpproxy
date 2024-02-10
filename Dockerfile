FROM rust:1.76.0

WORKDIR /src

COPY . .

RUN cargo install --path .

ENV URI=""
ENV TOKEN=""
ENV SERVICE_ID=""
ENV TIME_BEFORE_STOP=300
ENV TIME_TICK_CHECK_STOP=300
ENV TIME_WAIT_STATUS_CHANGE=400
ENV TIME_CHECK_ALREADY_STARTED=600

EXPOSE 35125/udp

CMD [ "udpproxy" ]

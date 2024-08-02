# Dev Notes

## Database Management

- $ sudo -i -u postgres
- $ psql
- $ create database ngao_event;
- $ SET ROLE ngao;
- $ SELECT CURRENT_USER;
- $ \c ngao_event

## Database Config

DATABASE_URL: postgres://ngao:<sudo_pass>@localhost/ngao_event

## Kafka Management

### Start the ZooKeeper service

$ bin/zookeeper-server-start.sh config/zookeeper.properties

### Start the Kafka broker service

$ bin/kafka-server-start.sh config/server.properties

### Creating topics

$ bin/kafka-topics.sh --create --topic quickstart-events --bootstrap-server localhost:9092
$ bin/kafka-topics.sh --create --if-not-exists --topic messages --bootstrap-server localhost:9092 --replication-factor 1 --partitions 1

### Setting up kafka using docker container

The following should be saved in docker-compose.yml

```yml
# docker-compose.yml
version: "3"

services:
# kafka
  zookeeper-1:
    container_name: zookeeper-1
    image: zookeeper
    restart: always
    ports:
      - 2181:2181
    environment:
      - ZOOKEEPER_CLIENT_PORT=2181
    volumes:
    - ./config/zookeeper-1/zookeeper.properties:/kafka/config/zookeeper.properties

  kafka-1:
    container_name: kafka-1
    image: bitnami/kafka
    restart: on-failure
    depends_on:
      - zookeeper-1
    ports:
      - 9092:9092
    environment:
      - KAFKA_ZOOKEEPER_CONNECT=zookeeper-1:2181
      - KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://localhost:9092
      - ALLOW_PLAINTEXT_LISTENER=yes
      - KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP=CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
      - KAFKA_AUTO_CREATE_TOPICS_ENABLE=true
      - KAFKA_CREATE_TOPICS=messages:1:3
    healthcheck:
      test: [ "CMD-SHELL", "kafka-topics.sh --bootstrap-server kafka:9092 --list" ]
      interval: 5s
      timeout: 10s
      retries: 5
networks:
  net:
    name: "net"
    driver: bridge
```

You may find on startup that the topic is not initialised properly, in which case you can use this shell snippet to auto-create a topic in the Docker container:

```sh
#!/usr/bin/env sh

docker exec kafka-1 /opt/bitnami/kafka/bin/kafka-topics.sh \
--bootstrap-server localhost:9092 --create --if-not-exists --topic messages \
--replication-factor 1 --partitions 1
```

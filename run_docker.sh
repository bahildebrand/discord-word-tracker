#!/bin/bash

docker run -d -e YOUTUBE_TOKEN=$YOUTUBE_TOKEN \
	-e DISCORD_TOKEN=$DISCORD_TOKEN \
	-e DB_PATH=$DB_PATH \
	discord_tracker

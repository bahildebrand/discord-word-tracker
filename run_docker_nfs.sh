#!/bin/bash

export NFS_VOL_NAME=shill_nfs
export NFS_LOCAL_MNT=/$DB_PATH
export NFS_SERVER=192.168.1.115
export NFS_SHARE=/ShillDB
export NFS_OPTS=vers=4,soft

docker run --mount \
	"src=$NFS_VOL_NAME,dst=$NFS_LOCAL_MNT,volume-opt=device=:$NFS_SHARE,\"volume-opt=o=addr=$NFS_SERVER,$NFS_OPTS\",type=volume,volume-driver=local,volume-opt=type=nfs" \
	-e YOUTUBE_TOKEN=$YOUTUBE_TOKEN \
	-e DISCORD_TOKEN=$DISCORD_TOKEN \
	-e DB_PATH=$DB_PATH \
	discord_tracker

#!/bin/bash
gcloud compute scp server/libnull_box.so sebjfk@nullbox-server-1:/home/sebjfk/nullbox_server
gcloud compute scp server/nullbox_server.pck sebjfk@nullbox-server-1:/home/sebjfk/nullbox_server
gcloud compute scp server/nullbox_server.x86_64 sebjfk@nullbox-server-1:/home/sebjfk/nullbox_server
gcloud compute ssh sebfjk@nullbox-server-1 --zone us-central1-a
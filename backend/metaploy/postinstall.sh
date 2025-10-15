#!/bin/sh

cleanup() {
	echo "Container stopped. Removing nginx configuration."
	rm /etc/nginx/sites-enabled/maintos.metaploy.conf
}

trap 'cleanup' SIGQUIT SIGTERM SIGHUP

"${@}" &

cp ./maintos.metaploy.conf /etc/nginx/sites-enabled

wait $!

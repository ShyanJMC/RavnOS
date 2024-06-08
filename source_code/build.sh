#!/bin/bash

if [ $# -eq 0 ]; then
	echo -e "No arguments supplied"
	echo -e "Arguments;\n all\n huginn (sysinit)\n kernel\n rune (shell and builtins)\n search\n"
	exit
fi

function kernel(){
	cd kernel/
	echo "Insert supported board;"
	read board
	bash build.sh $board
	cd ../
}

function huginn(){
	cargo build --bin huginn --target x86_64-unknown-linux-musl --release
	if [ $? -eq 0 ]; then
		strip target/x86_64-unknown-linux-musl/release/huginn
	fi
}

function rune(){
	cargo build --bin rune --target x86_64-unknown-linux-musl --release
	if [ $? -eq 0 ]; then
		trip target/x86_64-unknown-linux-musl/release/rune
	fi
}

function search(){
	cargo build --bin search --target x86_64-unknown-linux-musl --release
	if [ $? -eq 0 ]; then
		strip target/x86_64-unknown-linux-musl/release/search
	fi
}

if [ $1 == "all" ]; then
	kernel;
	huginn;
	rune;
	search;
elif [ $1 == "huginn" ]; then
	huginn;
elif [ $1 == "kernel" ]; then
	kernel;
elif [ $1 == "rune" ]; then
	rune;
elif [ $1 == "search"]; then
	search;
fi

#!/usr/bin/expect

set SERVER_NAME [lindex $argv 0]
set IP [lindex $argv 1]
set PORT [lindex $argv 2]
set USER_NAME [lindex $argv 3]
set PASSWORD [lindex $argv 4]

spawn ssh -p $PORT $USER_NAME@$IP

expect {
    -timeout 300
    "*assword" { send "$PASSWORD\r\n"; exp_continue ; sleep 3; }
    "yes/no" { send yes\n"; exp_continue; }
    "Last*" {
        puts "\nLogin Successful!!!\n";
    }
    timeout { puts "Expect was timeout."; return }
}

interact

#!/usr/bin/env ruby
STDOUT.sync = true

puts "Run process1 #{$$}"

loop do
  puts "Tick process1 #{$$}"
  sleep 5
end

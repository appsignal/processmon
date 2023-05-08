#!/usr/bin/env ruby
STDOUT.sync = true

puts "Run process2 #{$$}"

loop do
  puts "Tick process2 #{$$}"
  sleep 5
end

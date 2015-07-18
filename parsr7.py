'''parser7
    read line-by-line because I realized I'm an idiot
        alt is opening entire filee
        old version stored entire dump in 1 string (lol)
        I managed to do it with a ~100GB pagefile, but it became unbearably slow (of course)
    ideal arguments:
        --help  
        [1]     input file
        --output
        --update
        --chunks
        --chunk_size
        
    should use SHA hash instead of time-stamp
    maybe use a real argument parsing library
''' 

'''import argparse
parser = argparse.ArgumentParser(description="Extract link structure from Wikipedia dump")
parser.add_argument("input", type=string, )
args = parser.parse_args()'''

import datetime, sys, os, re
start_time = datetime.datetime.now()

#handle args
if len(sys.argv) == 2:
    #user-supplied input, generated output
    input_file = sys.argv[1]
    output_file = "out_%d.txt"
    output_num = 0
    while os.path.isfile(output_file % output_num):
        output_num += 1
    output_file = output_file % output_num
elif len(sys.argv) == 3:
    #user-supplied input and output
    input_file = sys.argv[1]
    output_file = sys.argv[2]
else:
    print "Usage: \""+sys.argv[0]+" input [output]\""
    exit()

lines = 0
with open(input_file) as input:
    for line in input:
        lines += 1

print "lines: ", lines
        
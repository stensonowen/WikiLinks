''' log:
segmentr1.py:
    breaks up large text file into smaller ones
    created after learning that decreasing text file size causes exponential parsing speedup
        on file misc_data\sample2\disk1.gsd (10% of sample english wiki, ~50 MB):
            parser5_2.py takes ~115 seconds on individual file,
            but only takes ~12 seconds on same file in 12 chunks (yet to be optimized)
    limited by python's ability to open large text files
        poor man's gsplit (http://www.gdgsoft.com/gsplit/)
    TODO: multiprocessing (maybe?)
    runs in <15 seconds on extracted simple wiki dump
segmentr2.py:
    breaks up files into distinct groups of pages, not just by bytes
    useful in case parser script must be called on multiple sections independently
    not very smart breaking: just uses a minimum size, so last file might only contain 1 byte
    still runs in <15 seconds on extracted simple wiki dump
    added command-line args
'''

import os, datetime, math, sys
now = datetime.datetime.now()
#sample dump:
#in_path = r"E:\Libraries\Programs\C++_RPI\WikiLinkr"
#in_file = "simplewiki-20150603-pages-articles.xml"

if len(sys.argv) == 1:
    in_path = "E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\out12"
    in_name = "disk1.gsd"
    in_file = os.path.join(in_path, in_name)
    #first 10% of sample dump:
    #in_path = r"E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\sample2"
    #in_file = "disk1.gsd"
    out_path = r"E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\out13"
    out_prefix = "output_"
    out_file = os.path.join(out_path, out_prefix)
else:
    in_file = sys.argv[1]
    out_file = sys.argv[2]
    out_path = out_file[:-out_file[::-1].index("\\")]

separator = "<page>"
fin = open(in_file, "r").read()

if not os.path.exists(out_path):
    os.makedirs(out_path)

#option1: calculate size of chunks from number
chunks = 12
chunk_size = len(fin)/chunks
#option2: calculate number of chunks from size
#chunk_size = 250*1024  #in bytes
#chunks = math.ceil(float(len(fin))/chunk_size)

log_total = len(str(chunks))    #determine length of number count; cf log_10(len)
chunk_start = 0
chunk_end = 0

for i in range(chunks):
    filename = out_file + str(i).zfill(log_total) + ".txt"    #pad index with zeros (to sort right (when order mattered))
    fout = open(filename, "w")
    
    chunk_start = chunk_end
    if separator in fin[chunk_start+chunk_size:]:
        #find index in master of end of this chunk (iff it's not the last section)
        chunk_end = fin.index(separator, chunk_start+chunk_size)
    else:
        #the last section can be shorter than the rest
        chunk_end = len(fin)
    
    fout.write(fin[chunk_start:chunk_end])

    fout.close()

print "just broke up \t...", in_file[len(in_file)/3:]
print "into " + str(chunks) + " pieces from \t...", out_file[len(out_file)*2/3:], "0.txt"
print "elapsed time: ", datetime.datetime.now() - now

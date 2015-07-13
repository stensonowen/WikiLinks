'''breaks up large text file into smaller ones
    created after learning that decreasing text file size causes exponential parsing speedup
    limited by python's ability to open large text files
        poor man's gsplit (http://www.gdgsoft.com/gsplit/)
    TODO: multiprocessing (maybe?)
    runs in <15 seconds on extracted simple wiki dump
'''

import os, datetime
now = datetime.datetime.now()

out_path = r"E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\out5"
chunks = 25



in_path = r"E:\Libraries\Programs\C++_RPI\WikiLinkr"
in_file = "simplewiki-20150603-pages-articles.xml"
out_prefix = "output_"


if not os.path.exists(out_path):
    os.makedirs(out_path)

fin = open(os.path.join(in_path, in_file), "r").read()
total_size = len(fin)
chunk_size = total_size/chunks
log_total = len(str(chunks))    #determine length of number count; cf log_10(len)

for i in range(chunks):
    filename = out_prefix + str(i).zfill(log_total) + ".txt"
    fout = open(os.path.join(out_path, filename), "w")
    if i != chunks-1:
        chunk = fin[i*chunk_size:(i+1)*chunk_size]
    else:
        chunk = fin[(chunks-1)*chunk_size:]
    fout.write(chunk)
    fout.close()
    
    

    
print "elapsed time: ", datetime.datetime.now() - now

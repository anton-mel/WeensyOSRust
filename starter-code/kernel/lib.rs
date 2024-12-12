// To write an operating system kernel, we need code that does not depend on any 
// operating system features. This means that we can’t use threads, files, heap 
// memory, the network, random numbers, standard output, or any other features 
// requiring OS abstractions or specific hardware. Which makes sense, since we’re 
// trying to write our own OS and our own drivers

// Create an "baremetal" executable that 
// can be run without an underlying OS
#![no_std]
#![no_main]

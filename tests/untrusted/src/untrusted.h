// untrusted/src/untrusted.h - PKRU-Safe
//
// Copyright 2018 Paul Kirth
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

#ifndef MPK_UNTRUSTED_H_
#define MPK_UNTRUSTED_H_

extern char* mybuff;
extern int global_val;
extern char* use_ptr(const char* ptr);
extern char* use_two_ptr(const char* charptr, int* intptr);
extern int change_vector(int* first, int* second);
extern int get_last_val(int* val_ptr);
extern int use_arc_array(const int* arc_array, unsigned size);
extern void access_vec(const int* array, unsigned size);

#endif

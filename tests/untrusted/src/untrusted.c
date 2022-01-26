// untrusted/src/untrusted.c - PKRU-Safe
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

#include "untrusted.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char my_buff[32];
int global_val;
#define OUTPUT 0

int change_vector(int *first, int *second) {
  int ret = first && second;
  if (ret) {
    int tmp = *first;
    *first = *second;
    *second = tmp;
  }
  return ret;
}

char *use_ptr(const char *ptr) {
  assert(ptr != NULL && "recieved NULL pointer to string");
  int len = strlen(ptr);
  char *buff = malloc(len);
  assert(buff != NULL && "malloc failed");
  assert(ptr != buff && "new buffer and origninal buffer have same address!!");
  memset(buff, 0, len);
  strcpy(buff, ptr);
  assert((0 == strcmp(ptr, buff)) &&
         "passed in buffer is not the same as our copy");
  /*printf("%s\n", ptr);*/
  /*return ptr;*/
  return buff; // return buffer to rust
}

char *use_two_ptr(const char *charptr, int *intptr) {
  assert(intptr != NULL && "recieved NULL pointer to integer");
  int c = *intptr;
  printf("Integer was: %d\n", c);
  *intptr = 0;
  return use_ptr(charptr);
}

int get_last_val(int *val_ptr) {
  int temp = global_val;
  if (val_ptr != NULL)
    global_val = *val_ptr;
  return temp;
}

int use_arc_array(const int *arc_array, unsigned size) {
  int sum = 0;
  size_t i = 0;
  for (i = 0; i < size; ++i) {
    sum += arc_array[i];
  }

  return sum;
}

void access_vec(const int *array, unsigned size) {
  if (array) {
#if OUTPUT
    printf("[");
    unsigned i, first;
    first = 1;
    for (i = 0; i < size; ++i) {

      if (first) {
        first = 0;
        printf("%d ", array[i]);
      } else {
        printf(", %d", array[i]);
      }
    }
    printf("];\n");
#else
    unsigned i;
    for (i = 0; i < size; ++i) {
      assert(array[i] == (int)i);
    }
#endif
  }
}

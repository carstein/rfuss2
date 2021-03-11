#include <stdio.h>
#include <stdlib.h>

void do_comparison(char* data) {
  if (data[0] == 'A') {
    if (data[1] == 'B') {
      if (data[2] == 'C') {
        if (data[3] == 'D') {
          if (data[4] == 'E') {
            if (data[5] == 'F') {
              // This is going to crash
              char* crash = 0;
              crash[0] = 'X';
            }
          }
        }
      }
    }
  }
}

int main(int argc, char* argv[]) {
  // parse arg
  if (argc < 2) {
    printf("Usage: %s <filename>\n", argv[0]);
    return -1;
  }

  // Open file
  FILE *fp;
  fp = fopen(argv[1], "r");
  
  // Get size
  fseek(fp, 0, SEEK_END);
  long size = ftell(fp);
  rewind(fp);

  // Read data from the file
  char* data = (char *)malloc(size * sizeof(char));
  fread(data, sizeof(char), size, fp);

  // Close file
  fclose(fp);

  // run comparison
  do_comparison(data);

  return 0;
}


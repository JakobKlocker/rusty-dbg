#include <stdio.h>
#include "printf.h"

int g_testing = 5;

int first();


int main(int argc, char **argv)
{
    first();
    return 1;
}

int multipl(int a, int b)
{
    return a * b;
}

void third(){
    printf("third");
}

void second(){
    printf("second");
    third();
}

int first()
{
    printf("hi");
    second();
    return 20;
}
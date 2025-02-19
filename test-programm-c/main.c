int g_testing = 5;

int main(int argc, char **argv)
{
    int hello = multipl(5, g_testing);
    return hello;
}

int multipl(int a, int b)
{
    return a * b;
}
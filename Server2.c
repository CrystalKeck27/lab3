/**********************************************
*  Filename: TCP Server.c
*  Description: Basic TCP comms
*  Author: Bob Turney
*  Date: 3/7/2024
*  Note: gcc -o Server Server.c
*        ./Client
*        ./Server
***********************************************/  
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <netdb.h>
#include <netinet/ip.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>

// define some constants 
#define MAX 256
#define SERVER_HOST "localhost"
#define SERVER_IP "192.168.136.1" //"127.0.0.1" $hostname -I
#define BUFLEN 256
#define SERVER_PORT 1234

char line[BUFLEN];
struct sockaddr_in server_addr, client_addr;
int mysock, csock, r, len, n;

int server_init();

int main (void)
{
  char line[MAX];
  server_init();
  while (1) //Try to accept a client request
  {
    printf("server: accepting new connection ...\n");
    len = sizeof(client_addr);
    csock = accept(mysock, (struct sockaddr *)&client_addr, &len);
    if (csock < 0)
    {
      printf("server accept error \n");
    }
    printf("server accepted a client connection from \n");
    //printf("Client: IP= %s port=%d", inet_ntoa(client_addr.sin_addr.s_addr), ntohs(client_addr.sin_port));
    printf("Client: IP= %s port=%d \n", inet_ntoa(client_addr.sin_addr), ntohs(client_addr.sin_port));
  
  while(1)
  {
    //n = read(csock, line, MAX);
    n = recvfrom(csock, line, MAX, 0, NULL, NULL);
    if (n == 0)
    {
      printf("server client died, server loops\n");
      close(csock);
      break;
    }
    // show the string
    printf("server read %s\n", line);
    // write back to client
    //n = write(csock, line, MAX);
    n = sendto(csock, line, MAX, 0, (struct sockaddr *)&client_addr, sizeof(client_addr));
    printf("server wrote %s\n", line);
    if (!strcmp(line,"quit")) return 0;
  }
  }
  return 0;
}

int server_init()
{
  printf("create stream socket\n");
  mysock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
  if (mysock < 0)
  {
    printf("socket call failed\n");
  }
  printf("fill server address with host IP and Port number\n");
  server_addr.sin_family = AF_INET;
  //inet_pton(AF_INET, SERVER_IP, &server_addr.sin_addr);
  server_addr.sin_addr.s_addr = htonl(INADDR_ANY);
  server_addr.sin_port = htons(SERVER_PORT);
  printf("bind the socket");
  r = bind(mysock, (struct sockaddr*)&server_addr, sizeof(server_addr));
  if (r<0)
  {
    printf("bind failed\n");
  }
  printf("server is listening  ...\n");
  listen(mysock, 5); // queue length of 5
  printf("server init done\n");
}
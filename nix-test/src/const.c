#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netinet/ip.h>
#include <netinet/ip6.h>
#include <netinet/tcp.h>
#include <netinet/udp.h>

#define GET_CONST(CONST)                \
    do {                                \
        if (0 == strcmp(err, #CONST)) { \
            return CONST;               \
        }                               \
    } while (0)

int
get_int_const(const char* err) {

    /*
     *
     * ===== ERRNO =====
     *
     */

    GET_CONST(EPERM);
    GET_CONST(ENOENT);
    GET_CONST(ESRCH);
    GET_CONST(EINTR);
    GET_CONST(EIO);
    GET_CONST(ENXIO);
    GET_CONST(E2BIG);
    GET_CONST(ENOEXEC);
    GET_CONST(EBADF);
    GET_CONST(ECHILD);
    GET_CONST(EAGAIN);
    GET_CONST(ENOMEM);
    GET_CONST(EACCES);
    GET_CONST(EFAULT);
    GET_CONST(ENOTBLK);
    GET_CONST(EBUSY);
    GET_CONST(EEXIST);
    GET_CONST(EXDEV);
    GET_CONST(ENODEV);
    GET_CONST(ENOTDIR);
    GET_CONST(EISDIR);
    GET_CONST(EINVAL);
    GET_CONST(ENFILE);
    GET_CONST(EMFILE);
    GET_CONST(ENOTTY);
    GET_CONST(ETXTBSY);
    GET_CONST(EFBIG);
    GET_CONST(ENOSPC);
    GET_CONST(ESPIPE);
    GET_CONST(EROFS);
    GET_CONST(EMLINK);
    GET_CONST(EPIPE);
    GET_CONST(EDOM);
    GET_CONST(ERANGE);
    GET_CONST(EDEADLK);
    GET_CONST(ENAMETOOLONG);
    GET_CONST(ENOLCK);
    GET_CONST(ENOSYS);
    GET_CONST(ENOTEMPTY);
    GET_CONST(ELOOP);
    GET_CONST(ENOMSG);
    GET_CONST(EIDRM);
    GET_CONST(EINPROGRESS);
    GET_CONST(EALREADY);
    GET_CONST(ENOTSOCK);
    GET_CONST(EDESTADDRREQ);
    GET_CONST(EMSGSIZE);
    GET_CONST(EPROTOTYPE);
    GET_CONST(ENOPROTOOPT);
    GET_CONST(EPROTONOSUPPORT);
    GET_CONST(ESOCKTNOSUPPORT);
    GET_CONST(EPFNOSUPPORT);
    GET_CONST(EAFNOSUPPORT);
    GET_CONST(EADDRINUSE);
    GET_CONST(EADDRNOTAVAIL);
    GET_CONST(ENETDOWN);
    GET_CONST(ENETUNREACH);
    GET_CONST(ENETRESET);
    GET_CONST(ECONNABORTED);
    GET_CONST(ECONNRESET);
    GET_CONST(ENOBUFS);
    GET_CONST(EISCONN);
    GET_CONST(ENOTCONN);
    GET_CONST(ESHUTDOWN);
    GET_CONST(ETOOMANYREFS);
    GET_CONST(ETIMEDOUT);
    GET_CONST(ECONNREFUSED);
    GET_CONST(EHOSTDOWN);
    GET_CONST(EHOSTUNREACH);

#ifdef LINUX
    GET_CONST(ECHRNG);
    GET_CONST(EL2NSYNC);
    GET_CONST(EL3HLT);
    GET_CONST(EL3RST);
    GET_CONST(ELNRNG);
    GET_CONST(EUNATCH);
    GET_CONST(ENOCSI);
    GET_CONST(EL2HLT);
    GET_CONST(EBADE);
    GET_CONST(EBADR);
    GET_CONST(EXFULL);
    GET_CONST(ENOANO);
    GET_CONST(EBADRQC);
    GET_CONST(EBADSLT);
    GET_CONST(EBFONT);
    GET_CONST(ENOSTR);
    GET_CONST(ENODATA);
    GET_CONST(ETIME);
    GET_CONST(ENOSR);
    GET_CONST(ENONET);
    GET_CONST(ENOPKG);
    GET_CONST(EREMOTE);
    GET_CONST(ENOLINK);
    GET_CONST(EADV);
    GET_CONST(ESRMNT);
    GET_CONST(ECOMM);
    GET_CONST(EPROTO);
    GET_CONST(EMULTIHOP);
    GET_CONST(EDOTDOT);
    GET_CONST(EBADMSG);
    GET_CONST(EOVERFLOW);
    GET_CONST(ENOTUNIQ);
    GET_CONST(EBADFD);
    GET_CONST(EREMCHG);
    GET_CONST(ELIBACC);
    GET_CONST(ELIBBAD);
    GET_CONST(ELIBSCN);
    GET_CONST(ELIBMAX);
    GET_CONST(ELIBEXEC);
    GET_CONST(EILSEQ);
    GET_CONST(ERESTART);
    GET_CONST(ESTRPIPE);
    GET_CONST(EUSERS);
    GET_CONST(EOPNOTSUPP);
    GET_CONST(ESTALE);
    GET_CONST(EUCLEAN);
    GET_CONST(ENOTNAM);
    GET_CONST(ENAVAIL);
    GET_CONST(EISNAM);
    GET_CONST(EREMOTEIO);
    GET_CONST(EDQUOT);
    GET_CONST(ENOMEDIUM);
    GET_CONST(EMEDIUMTYPE);
    GET_CONST(ECANCELED);
    GET_CONST(ENOKEY);
    GET_CONST(EKEYEXPIRED);
    GET_CONST(EKEYREVOKED);
    GET_CONST(EKEYREJECTED);
    GET_CONST(EOWNERDEAD);
    GET_CONST(ENOTRECOVERABLE);
#ifndef __ANDROID__
    GET_CONST(ERFKILL);
    // GET_CONST(EHWPOISON);
#endif
#endif

#ifdef DARWIN
    GET_CONST(ENOTSUP);
    GET_CONST(EPROCLIM);
    GET_CONST(EUSERS);
    GET_CONST(EDQUOT);
    GET_CONST(ESTALE);
    GET_CONST(EREMOTE);
    GET_CONST(EBADRPC);
    GET_CONST(ERPCMISMATCH);
    GET_CONST(EPROGUNAVAIL);
    GET_CONST(EPROGMISMATCH);
    GET_CONST(EPROCUNAVAIL);
    GET_CONST(EFTYPE);
    GET_CONST(EAUTH);
    GET_CONST(ENEEDAUTH);
    GET_CONST(EPWROFF);
    GET_CONST(EDEVERR);
    GET_CONST(EOVERFLOW);
    GET_CONST(EBADEXEC);
    GET_CONST(EBADARCH);
    GET_CONST(ESHLIBVERS);
    GET_CONST(EBADMACHO);
    GET_CONST(ECANCELED);
    GET_CONST(EILSEQ);
    GET_CONST(ENOATTR);
    GET_CONST(EBADMSG);
    GET_CONST(EMULTIHOP);
    GET_CONST(ENODATA);
    GET_CONST(ENOLINK);
    GET_CONST(ENOSR);
    GET_CONST(ENOSTR);
    GET_CONST(EPROTO);
    GET_CONST(ETIME);
    GET_CONST(EOPNOTSUPP);
    GET_CONST(ENOPOLICY);
    GET_CONST(ENOTRECOVERABLE);
    GET_CONST(EOWNERDEAD);
    GET_CONST(EQFULL);
#endif

    /*
     *
     * ===== SOCKET OPTIONS =====
     *
     */

    GET_CONST(AF_UNIX);
    GET_CONST(AF_LOCAL);
    GET_CONST(AF_INET);
    GET_CONST(AF_INET6);
    GET_CONST(SOCK_STREAM);
    GET_CONST(SOCK_DGRAM);
    GET_CONST(SOCK_SEQPACKET);
    GET_CONST(SOCK_RAW);
    GET_CONST(SOCK_RDM);
    GET_CONST(SOL_SOCKET);
    GET_CONST(IPPROTO_IP);
    GET_CONST(IPPROTO_IPV6);
    GET_CONST(IPPROTO_TCP);
    GET_CONST(IPPROTO_UDP);
    GET_CONST(SO_ACCEPTCONN);
    GET_CONST(SO_BROADCAST);
    GET_CONST(SO_DEBUG);
    GET_CONST(SO_ERROR);
    GET_CONST(SO_DONTROUTE);
    GET_CONST(SO_KEEPALIVE);
    GET_CONST(SO_LINGER);
    GET_CONST(SO_OOBINLINE);
    GET_CONST(SO_RCVBUF);
    GET_CONST(SO_RCVLOWAT);
    GET_CONST(SO_SNDLOWAT);
    GET_CONST(SO_RCVTIMEO);
    GET_CONST(SO_SNDTIMEO);
    GET_CONST(SO_REUSEADDR);
    // GET_CONST(SO_REUSEPORT);
    GET_CONST(SO_SNDBUF);
    GET_CONST(SO_TIMESTAMP);
    GET_CONST(SO_TYPE);
    GET_CONST(TCP_NODELAY);
    GET_CONST(TCP_MAXSEG);
    GET_CONST(IP_MULTICAST_IF);
    GET_CONST(IP_MULTICAST_TTL);
    GET_CONST(IP_MULTICAST_LOOP);
    GET_CONST(IP_ADD_MEMBERSHIP);
    GET_CONST(IP_DROP_MEMBERSHIP);
    GET_CONST(INADDR_ANY);
    GET_CONST(INADDR_NONE);
    GET_CONST(INADDR_BROADCAST);
    GET_CONST(MSG_OOB);
    GET_CONST(MSG_PEEK);
    GET_CONST(MSG_DONTWAIT);
    GET_CONST(SHUT_RD);
    GET_CONST(SHUT_WR);
    GET_CONST(SHUT_RDWR);

#ifdef LINUX
    GET_CONST(SOL_IP);
    GET_CONST(SOL_TCP);
    GET_CONST(SOL_IPV6);
    GET_CONST(SOL_UDP);
    GET_CONST(SO_BINDTODEVICE);
    GET_CONST(SO_BSDCOMPAT);
    // GET_CONST(SO_DOMAIN);
    // GET_CONST(SO_MARK);
    GET_CONST(TCP_CORK);
    // GET_CONST(SO_BUSY_POLL);
    // GET_CONST(SO_RXQ_OVFL);
    GET_CONST(SO_PASSCRED);
    GET_CONST(SO_PRIORITY);
    // GET_CONST(SO_PROTOCOL);
    GET_CONST(SO_RCVBUFFORCE);
    // GET_CONST(SO_PEEK_OFF);
    GET_CONST(SO_PEERCRED);
    GET_CONST(SO_SNDBUFFORCE);
#endif

    return -1;

}

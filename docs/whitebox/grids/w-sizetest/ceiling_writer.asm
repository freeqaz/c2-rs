10b5e4cc:	8b 0d 98 ea c2 10    	mov    ecx,DWORD PTR ds:0x10c2ea98
10b5e4d2:	83 f9 06             	cmp    ecx,0x6
10b5e4d5:	7e 0c                	jle    0x10b5e4e3
10b5e4d7:	c7 05 18 63 c4 10 e8 	mov    DWORD PTR ds:0x10c46318,0x3e8
10b5e4de:	03 00 00 
10b5e4e1:	eb 0a                	jmp    0x10b5e4ed
10b5e4e3:	6a 10                	push   0x10
10b5e4e5:	58                   	pop    eax
10b5e4e6:	d3 e0                	shl    eax,cl
10b5e4e8:	a3 18 63 c4 10       	mov    ds:0x10c46318,eax
10b5e4ed:	e8 7f d5 ff ff       	call   0x10b5ba71
10b5e4f2:	e8 77 d7 ff ff       	call   0x10b5bc6e
10b5e4f7:	83 3d c4 62 c4 10 00 	cmp    DWORD PTR ds:0x10c462c4,0x0
10b5e4fe:	74 2e                	je     0x10b5e52e
10b5e500:	56                   	push   esi
10b5e501:	57                   	push   edi
10b5e502:	ff 74 24 10          	push   DWORD PTR [esp+0x10]
10b5e506:	ff 74 24 10          	push   DWORD PTR [esp+0x10]
10b5e50a:	e8 cf d4 ff ff       	call   0x10b5b9de
10b5e50f:	83 3d c8 f1 c6 10 00 	cmp    DWORD PTR ds:0x10c6f1c8,0x0
10b5e516:	6a 2e                	push   0x2e
10b5e518:	bf 10 f5 c3 10       	mov    edi,0x10c3f510
10b5e51d:	59                   	pop    ecx
10b5e51e:	be d0 5e c4 10       	mov    esi,0x10c45ed0
10b5e523:	75 05                	jne    0x10b5e52a
10b5e525:	be 18 5e c4 10       	mov    esi,0x10c45e18
10b5e52a:	f3 a5                	rep movs DWORD PTR es:[edi],DWORD PTR ds:[esi]
10b5e52c:	5f                   	pop    edi
10b5e52d:	5e                   	pop    esi
10b5e52e:	c2 08 00             	ret    0x8

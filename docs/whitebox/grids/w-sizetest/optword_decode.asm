10b82338:	8b 41 1c             	mov    eax,DWORD PTR [ecx+0x1c]
10b8233b:	57                   	push   edi
10b8233c:	33 ff                	xor    edi,edi
10b8233e:	39 3d ac ea c2 10    	cmp    DWORD PTR ds:0x10c2eaac,edi
10b82344:	74 0f                	je     0x10b82355
10b82346:	8b 09                	mov    ecx,DWORD PTR [ecx]
10b82348:	8b 89 80 00 00 00    	mov    ecx,DWORD PTR [ecx+0x80]
10b8234e:	3b cf                	cmp    ecx,edi
10b82350:	74 03                	je     0x10b82355
10b82352:	89 41 76             	mov    DWORD PTR [ecx+0x76],eax
10b82355:	56                   	push   esi
10b82356:	33 f6                	xor    esi,esi
10b82358:	8b c8                	mov    ecx,eax
10b8235a:	c1 e9 08             	shr    ecx,0x8
10b8235d:	46                   	inc    esi
10b8235e:	23 ce                	and    ecx,esi
10b82360:	89 0d 14 e3 c2 10    	mov    DWORD PTR ds:0x10c2e314,ecx
10b82366:	8b c8                	mov    ecx,eax
10b82368:	c1 e9 1f             	shr    ecx,0x1f
10b8236b:	83 3d 20 de c3 10 02 	cmp    DWORD PTR ds:0x10c3de20,0x2
10b82372:	89 0d 18 e3 c2 10    	mov    DWORD PTR ds:0x10c2e318,ecx
10b82378:	74 20                	je     0x10b8239a
10b8237a:	39 3d ac ea c2 10    	cmp    DWORD PTR ds:0x10c2eaac,edi
10b82380:	74 09                	je     0x10b8238b
10b82382:	83 3d c8 f1 c6 10 02 	cmp    DWORD PTR ds:0x10c6f1c8,0x2
10b82389:	74 0f                	je     0x10b8239a
10b8238b:	8b c8                	mov    ecx,eax
10b8238d:	c1 e9 17             	shr    ecx,0x17
10b82390:	23 ce                	and    ecx,esi
10b82392:	89 0d 10 e3 c2 10    	mov    DWORD PTR ds:0x10c2e310,ecx
10b82398:	eb 0d                	jmp    0x10b823a7
10b8239a:	8b c8                	mov    ecx,eax
10b8239c:	c1 e9 17             	shr    ecx,0x17
10b8239f:	23 ce                	and    ecx,esi
10b823a1:	89 0d dc dd c3 10    	mov    DWORD PTR ds:0x10c3dddc,ecx
10b823a7:	8b c8                	mov    ecx,eax
10b823a9:	c1 e9 12             	shr    ecx,0x12
10b823ac:	23 ce                	and    ecx,esi
10b823ae:	89 0d 0c e3 c2 10    	mov    DWORD PTR ds:0x10c2e30c,ecx
10b823b4:	75 0d                	jne    0x10b823c3
10b823b6:	8b c8                	mov    ecx,eax
10b823b8:	c1 e9 0c             	shr    ecx,0xc
10b823bb:	23 ce                	and    ecx,esi
10b823bd:	89 0d 0c e3 c2 10    	mov    DWORD PTR ds:0x10c2e30c,ecx
10b823c3:	b9 00 00 20 00       	mov    ecx,0x200000
10b823c8:	39 3d 68 cf c3 10    	cmp    DWORD PTR ds:0x10c3cf68,edi
10b823ce:	74 0a                	je     0x10b823da
10b823d0:	89 35 08 e3 c2 10    	mov    DWORD PTR ds:0x10c2e308,esi
10b823d6:	85 c1                	test   ecx,eax
10b823d8:	75 06                	jne    0x10b823e0
10b823da:	89 3d 08 e3 c2 10    	mov    DWORD PTR ds:0x10c2e308,edi
10b823e0:	8b d0                	mov    edx,eax
10b823e2:	c1 ea 15             	shr    edx,0x15
10b823e5:	23 d6                	and    edx,esi
10b823e7:	89 15 fc e2 c2 10    	mov    DWORD PTR ds:0x10c2e2fc,edx
10b823ed:	39 3d ac ea c2 10    	cmp    DWORD PTR ds:0x10c2eaac,edi

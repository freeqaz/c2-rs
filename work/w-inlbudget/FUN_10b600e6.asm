10b600e6:	55                   	push   ebp
10b600e7:	8b ec                	mov    ebp,esp
10b600e9:	83 ec 40             	sub    esp,0x40
10b600ec:	83 4d c0 ff          	or     DWORD PTR [ebp-0x40],0xffffffff
10b600f0:	83 4d c4 ff          	or     DWORD PTR [ebp-0x3c],0xffffffff
10b600f4:	53                   	push   ebx
10b600f5:	33 db                	xor    ebx,ebx
10b600f7:	89 1a                	mov    DWORD PTR [edx],ebx
10b600f9:	8b 41 08             	mov    eax,DWORD PTR [ecx+0x8]
10b600fc:	8b 00                	mov    eax,DWORD PTR [eax]
10b600fe:	56                   	push   esi
10b600ff:	8b 70 1c             	mov    esi,DWORD PTR [eax+0x1c]
10b60102:	89 55 c8             	mov    DWORD PTR [ebp-0x38],edx
10b60105:	89 4d ec             	mov    DWORD PTR [ebp-0x14],ecx
10b60108:	89 5d d4             	mov    DWORD PTR [ebp-0x2c],ebx
10b6010b:	89 5d dc             	mov    DWORD PTR [ebp-0x24],ebx
10b6010e:	89 5d fc             	mov    DWORD PTR [ebp-0x4],ebx
10b60111:	89 5d f4             	mov    DWORD PTR [ebp-0xc],ebx
10b60114:	89 5d f8             	mov    DWORD PTR [ebp-0x8],ebx
10b60117:	89 5d d0             	mov    DWORD PTR [ebp-0x30],ebx
10b6011a:	89 5d f0             	mov    DWORD PTR [ebp-0x10],ebx
10b6011d:	89 5d d8             	mov    DWORD PTR [ebp-0x28],ebx
10b60120:	89 5d e4             	mov    DWORD PTR [ebp-0x1c],ebx
10b60123:	89 5d e8             	mov    DWORD PTR [ebp-0x18],ebx
10b60126:	89 5d e0             	mov    DWORD PTR [ebp-0x20],ebx
10b60129:	3b f3                	cmp    esi,ebx
10b6012b:	0f 84 8e 03 00 00    	je     0x10b604bf
10b60131:	57                   	push   edi
10b60132:	eb 03                	jmp    0x10b60137
10b60134:	8b 4d ec             	mov    ecx,DWORD PTR [ebp-0x14]
10b60137:	0f b7 46 14          	movzx  eax,WORD PTR [esi+0x14]
10b6013b:	66 3b c3             	cmp    ax,bx
10b6013e:	74 11                	je     0x10b60151
10b60140:	a3 ec e2 c2 10       	mov    ds:0x10c2e2ec,eax
10b60145:	0f b7 46 14          	movzx  eax,WORD PTR [esi+0x14]
10b60149:	03 41 44             	add    eax,DWORD PTR [ecx+0x44]
10b6014c:	a3 e0 e2 c2 10       	mov    ds:0x10c2e2e0,eax
10b60151:	f6 46 09 01          	test   BYTE PTR [esi+0x9],0x1
10b60155:	8b 0e                	mov    ecx,DWORD PTR [esi]
10b60157:	89 4d cc             	mov    DWORD PTR [ebp-0x34],ecx
10b6015a:	0f 84 e6 02 00 00    	je     0x10b60446
10b60160:	8b 7e 28             	mov    edi,DWORD PTR [esi+0x28]
10b60163:	ff 45 d8             	inc    DWORD PTR [ebp-0x28]
10b60166:	8b c7                	mov    eax,edi
10b60168:	eb 18                	jmp    0x10b60182
10b6016a:	8a 48 08             	mov    cl,BYTE PTR [eax+0x8]
10b6016d:	42                   	inc    edx
10b6016e:	d3 e2                	shl    edx,cl
10b60170:	f6 c2 06             	test   dl,0x6
10b60173:	74 0b                	je     0x10b60180
10b60175:	8b 48 18             	mov    ecx,DWORD PTR [eax+0x18]
10b60178:	39 59 14             	cmp    DWORD PTR [ecx+0x14],ebx
10b6017b:	74 03                	je     0x10b60180
10b6017d:	ff 4d fc             	dec    DWORD PTR [ebp-0x4]
10b60180:	8b 00                	mov    eax,DWORD PTR [eax]
10b60182:	33 d2                	xor    edx,edx
10b60184:	3b c3                	cmp    eax,ebx
10b60186:	75 e2                	jne    0x10b6016a
10b60188:	66 8b 45 fc          	mov    ax,WORD PTR [ebp-0x4]
10b6018c:	66 89 46 1c          	mov    WORD PTR [esi+0x1c],ax
10b60190:	8b 46 2c             	mov    eax,DWORD PTR [esi+0x2c]
10b60193:	3b c3                	cmp    eax,ebx
10b60195:	74 28                	je     0x10b601bf
10b60197:	8a 48 08             	mov    cl,BYTE PTR [eax+0x8]
10b6019a:	33 db                	xor    ebx,ebx
10b6019c:	43                   	inc    ebx
10b6019d:	d3 e3                	shl    ebx,cl
10b6019f:	f6 c3 06             	test   bl,0x6
10b601a2:	74 0f                	je     0x10b601b3
10b601a4:	8b 48 18             	mov    ecx,DWORD PTR [eax+0x18]
10b601a7:	83 79 14 00          	cmp    DWORD PTR [ecx+0x14],0x0
10b601ab:	74 06                	je     0x10b601b3
10b601ad:	ff 45 fc             	inc    DWORD PTR [ebp-0x4]
10b601b0:	33 d2                	xor    edx,edx
10b601b2:	42                   	inc    edx
10b601b3:	8b 00                	mov    eax,DWORD PTR [eax]
10b601b5:	33 db                	xor    ebx,ebx
10b601b7:	3b c3                	cmp    eax,ebx
10b601b9:	75 dc                	jne    0x10b60197
10b601bb:	3b d3                	cmp    edx,ebx
10b601bd:	75 49                	jne    0x10b60208
10b601bf:	33 db                	xor    ebx,ebx
10b601c1:	39 5d e4             	cmp    DWORD PTR [ebp-0x1c],ebx
10b601c4:	74 36                	je     0x10b601fc
10b601c6:	80 7e 08 12          	cmp    BYTE PTR [esi+0x8],0x12
10b601ca:	75 1f                	jne    0x10b601eb
10b601cc:	39 1f                	cmp    DWORD PTR [edi],ebx
10b601ce:	74 22                	je     0x10b601f2
10b601d0:	8b 07                	mov    eax,DWORD PTR [edi]
10b601d2:	8a 48 08             	mov    cl,BYTE PTR [eax+0x8]
10b601d5:	33 d2                	xor    edx,edx
10b601d7:	42                   	inc    edx
10b601d8:	d3 e2                	shl    edx,cl
10b601da:	f6 c2 06             	test   dl,0x6
10b601dd:	74 13                	je     0x10b601f2
10b601df:	8b 40 18             	mov    eax,DWORD PTR [eax+0x18]
10b601e2:	8b 48 14             	mov    ecx,DWORD PTR [eax+0x14]
10b601e5:	3b cb                	cmp    ecx,ebx
10b601e7:	74 09                	je     0x10b601f2
10b601e9:	eb 02                	jmp    0x10b601ed
10b601eb:	8b ce                	mov    ecx,esi
10b601ed:	e8 29 7a 06 00       	call   0x10bc7c1b
10b601f2:	39 5d fc             	cmp    DWORD PTR [ebp-0x4],ebx
10b601f5:	75 11                	jne    0x10b60208
10b601f7:	89 5d e4             	mov    DWORD PTR [ebp-0x1c],ebx
10b601fa:	eb 0c                	jmp    0x10b60208
10b601fc:	39 5d fc             	cmp    DWORD PTR [ebp-0x4],ebx
10b601ff:	7e 07                	jle    0x10b60208
10b60201:	c7 45 e4 01 00 00 00 	mov    DWORD PTR [ebp-0x1c],0x1
10b60208:	8a 46 08             	mov    al,BYTE PTR [esi+0x8]
10b6020b:	3c 0f                	cmp    al,0xf
10b6020d:	0f 85 b7 01 00 00    	jne    0x10b603ca
10b60213:	0f b7 46 14          	movzx  eax,WORD PTR [esi+0x14]
10b60217:	66 3b c3             	cmp    ax,bx
10b6021a:	74 14                	je     0x10b60230
10b6021c:	8b 4d ec             	mov    ecx,DWORD PTR [ebp-0x14]
10b6021f:	a3 ec e2 c2 10       	mov    ds:0x10c2e2ec,eax
10b60224:	0f b7 46 14          	movzx  eax,WORD PTR [esi+0x14]
10b60228:	03 41 44             	add    eax,DWORD PTR [ecx+0x44]
10b6022b:	a3 e0 e2 c2 10       	mov    ds:0x10c2e2e0,eax
10b60230:	8b ce                	mov    ecx,esi
10b60232:	e8 55 ca ff ff       	call   0x10b5cc8c
10b60237:	85 c0                	test   eax,eax
10b60239:	0f 84 72 02 00 00    	je     0x10b604b1
10b6023f:	f6 46 34 01          	test   BYTE PTR [esi+0x34],0x1
10b60243:	0f 85 68 02 00 00    	jne    0x10b604b1
10b60249:	8b ce                	mov    ecx,esi
10b6024b:	e8 5e bd ff ff       	call   0x10b5bfae
10b60250:	8b 55 ec             	mov    edx,DWORD PTR [ebp-0x14]
10b60253:	6a 00                	push   0x0
10b60255:	ff 75 dc             	push   DWORD PTR [ebp-0x24]
10b60258:	8b d8                	mov    ebx,eax
10b6025a:	ff 75 08             	push   DWORD PTR [ebp+0x8]
10b6025d:	e8 fd f8 ff ff       	call   0x10b5fb5f
10b60262:	85 c0                	test   eax,eax
10b60264:	0f 85 bb 00 00 00    	jne    0x10b60325
10b6026a:	39 05 08 e3 c2 10    	cmp    DWORD PTR ds:0x10c2e308,eax
10b60270:	74 30                	je     0x10b602a2
10b60272:	8b 8b 80 00 00 00    	mov    ecx,DWORD PTR [ebx+0x80]
10b60278:	85 c9                	test   ecx,ecx
10b6027a:	74 26                	je     0x10b602a2
10b6027c:	8b 51 04             	mov    edx,DWORD PTR [ecx+0x4]
10b6027f:	3b da                	cmp    ebx,edx
10b60281:	74 11                	je     0x10b60294
10b60283:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b6028a:	75 16                	jne    0x10b602a2
10b6028c:	39 93 90 00 00 00    	cmp    DWORD PTR [ebx+0x90],edx
10b60292:	75 0e                	jne    0x10b602a2
10b60294:	8b 89 b1 00 00 00    	mov    ecx,DWORD PTR [ecx+0xb1]
10b6029a:	c1 e9 0a             	shr    ecx,0xa
10b6029d:	83 e1 01             	and    ecx,0x1
10b602a0:	eb 02                	jmp    0x10b602a4
10b602a2:	33 c9                	xor    ecx,ecx
10b602a4:	85 c9                	test   ecx,ecx
10b602a6:	74 7d                	je     0x10b60325
10b602a8:	33 ff                	xor    edi,edi
10b602aa:	47                   	inc    edi
10b602ab:	6a 20                	push   0x20
10b602ad:	5a                   	pop    edx
10b602ae:	6a 07                	push   0x7
10b602b0:	59                   	pop    ecx
10b602b1:	e8 74 ff 0b 00       	call   0x10c2022a
10b602b6:	8b 4d dc             	mov    ecx,DWORD PTR [ebp-0x24]
10b602b9:	89 48 08             	mov    DWORD PTR [eax+0x8],ecx
10b602bc:	33 c9                	xor    ecx,ecx
10b602be:	39 4d f0             	cmp    DWORD PTR [ebp-0x10],ecx
10b602c1:	89 70 04             	mov    DWORD PTR [eax+0x4],esi
10b602c4:	0f 95 c1             	setne  cl
10b602c7:	33 d2                	xor    edx,edx
10b602c9:	83 e7 01             	and    edi,0x1
10b602cc:	03 ff                	add    edi,edi
10b602ce:	c6 40 18 01          	mov    BYTE PTR [eax+0x18],0x1
10b602d2:	83 e1 01             	and    ecx,0x1
10b602d5:	0b cf                	or     ecx,edi
10b602d7:	03 c9                	add    ecx,ecx
10b602d9:	39 55 d0             	cmp    DWORD PTR [ebp-0x30],edx
10b602dc:	0f 95 c2             	setne  dl
10b602df:	83 e2 01             	and    edx,0x1
10b602e2:	0b ca                	or     ecx,edx
10b602e4:	33 d2                	xor    edx,edx
10b602e6:	c1 e1 02             	shl    ecx,0x2
10b602e9:	39 55 f4             	cmp    DWORD PTR [ebp-0xc],edx
10b602ec:	0f 95 c2             	setne  dl
10b602ef:	83 e2 01             	and    edx,0x1
10b602f2:	0b ca                	or     ecx,edx
10b602f4:	8b 50 1c             	mov    edx,DWORD PTR [eax+0x1c]
10b602f7:	83 e2 e2             	and    edx,0xffffffe2
10b602fa:	0b ca                	or     ecx,edx
10b602fc:	83 7d f8 00          	cmp    DWORD PTR [ebp-0x8],0x0
10b60300:	89 48 1c             	mov    DWORD PTR [eax+0x1c],ecx
10b60303:	74 34                	je     0x10b60339
10b60305:	8b 55 ec             	mov    edx,DWORD PTR [ebp-0x14]
10b60308:	f6 82 94 00 00 00 08 	test   BYTE PTR [edx+0x94],0x8
10b6030f:	74 06                	je     0x10b60317
10b60311:	83 7d 08 00          	cmp    DWORD PTR [ebp+0x8],0x0
10b60315:	74 22                	je     0x10b60339
10b60317:	f7 43 4c 00 20 00 00 	test   DWORD PTR [ebx+0x4c],0x2000
10b6031e:	75 19                	jne    0x10b60339
10b60320:	33 d2                	xor    edx,edx
10b60322:	42                   	inc    edx
10b60323:	eb 16                	jmp    0x10b6033b
10b60325:	33 ff                	xor    edi,edi
10b60327:	85 c0                	test   eax,eax
10b60329:	75 80                	jne    0x10b602ab
10b6032b:	8b d6                	mov    edx,esi
10b6032d:	8b cb                	mov    ecx,ebx
10b6032f:	e8 8c bc ff ff       	call   0x10b5bfc0
10b60334:	e9 78 01 00 00       	jmp    0x10b604b1
10b60339:	33 d2                	xor    edx,edx
10b6033b:	03 d2                	add    edx,edx
10b6033d:	33 d1                	xor    edx,ecx
10b6033f:	83 e2 02             	and    edx,0x2
10b60342:	33 d1                	xor    edx,ecx
10b60344:	8b 4d c0             	mov    ecx,DWORD PTR [ebp-0x40]
10b60347:	89 50 1c             	mov    DWORD PTR [eax+0x1c],edx
10b6034a:	8b 55 c4             	mov    edx,DWORD PTR [ebp-0x3c]
10b6034d:	8b f1                	mov    esi,ecx
10b6034f:	23 f2                	and    esi,edx
10b60351:	83 fe ff             	cmp    esi,0xffffffff
10b60354:	75 04                	jne    0x10b6035a
10b60356:	33 c9                	xor    ecx,ecx
10b60358:	33 d2                	xor    edx,edx
10b6035a:	89 48 10             	mov    DWORD PTR [eax+0x10],ecx
10b6035d:	8b 4d e0             	mov    ecx,DWORD PTR [ebp-0x20]
10b60360:	89 50 14             	mov    DWORD PTR [eax+0x14],edx
10b60363:	89 45 e0             	mov    DWORD PTR [ebp-0x20],eax
10b60366:	85 c9                	test   ecx,ecx
10b60368:	75 05                	jne    0x10b6036f
10b6036a:	89 45 e8             	mov    DWORD PTR [ebp-0x18],eax
10b6036d:	eb 02                	jmp    0x10b60371
10b6036f:	89 01                	mov    DWORD PTR [ecx],eax
10b60371:	8b 45 c8             	mov    eax,DWORD PTR [ebp-0x38]
10b60374:	ff 00                	inc    DWORD PTR [eax]
10b60376:	f6 43 4c 10          	test   BYTE PTR [ebx+0x4c],0x10
10b6037a:	0f 84 31 01 00 00    	je     0x10b604b1
10b60380:	8b 75 d4             	mov    esi,DWORD PTR [ebp-0x2c]
10b60383:	8b c6                	mov    eax,esi
10b60385:	85 f6                	test   esi,esi
10b60387:	74 25                	je     0x10b603ae
10b60389:	39 58 04             	cmp    DWORD PTR [eax+0x4],ebx
10b6038c:	74 06                	je     0x10b60394
10b6038e:	8b 00                	mov    eax,DWORD PTR [eax]
10b60390:	85 c0                	test   eax,eax
10b60392:	75 f5                	jne    0x10b60389
10b60394:	85 c0                	test   eax,eax
10b60396:	74 16                	je     0x10b603ae
10b60398:	8a 48 08             	mov    cl,BYTE PTR [eax+0x8]
10b6039b:	80 f9 ff             	cmp    cl,0xff
10b6039e:	0f 83 0d 01 00 00    	jae    0x10b604b1
10b603a4:	fe c1                	inc    cl
10b603a6:	88 48 08             	mov    BYTE PTR [eax+0x8],cl
10b603a9:	e9 03 01 00 00       	jmp    0x10b604b1
10b603ae:	6a 0c                	push   0xc
10b603b0:	5a                   	pop    edx
10b603b1:	6a 07                	push   0x7
10b603b3:	59                   	pop    ecx
10b603b4:	e8 71 fe 0b 00       	call   0x10c2022a
10b603b9:	89 58 04             	mov    DWORD PTR [eax+0x4],ebx
10b603bc:	c6 40 08 01          	mov    BYTE PTR [eax+0x8],0x1
10b603c0:	89 30                	mov    DWORD PTR [eax],esi
10b603c2:	89 45 d4             	mov    DWORD PTR [ebp-0x2c],eax
10b603c5:	e9 e7 00 00 00       	jmp    0x10b604b1
10b603ca:	3c 16                	cmp    al,0x16
10b603cc:	75 16                	jne    0x10b603e4
10b603ce:	81 7e 04 ed 02 00 00 	cmp    DWORD PTR [esi+0x4],0x2ed
10b603d5:	75 0d                	jne    0x10b603e4
10b603d7:	8b 4e 34             	mov    ecx,DWORD PTR [esi+0x34]
10b603da:	e8 ff d5 ff ff       	call   0x10b5d9de
10b603df:	e9 cd 00 00 00       	jmp    0x10b604b1
10b603e4:	3c 15                	cmp    al,0x15
10b603e6:	0f 85 c5 00 00 00    	jne    0x10b604b1
10b603ec:	8b 46 04             	mov    eax,DWORD PTR [esi+0x4]
10b603ef:	8d 88 12 fd ff ff    	lea    ecx,[eax-0x2ee]
10b603f5:	83 f9 12             	cmp    ecx,0x12
10b603f8:	0f 87 b3 00 00 00    	ja     0x10b604b1
10b603fe:	0f b6 89 22 05 b6 10 	movzx  ecx,BYTE PTR [ecx+0x10b60522]
10b60405:	ff 24 8d 0e 05 b6 10 	jmp    DWORD PTR [ecx*4+0x10b6050e]
10b6040c:	ff 45 f8             	inc    DWORD PTR [ebp-0x8]
10b6040f:	e9 9d 00 00 00       	jmp    0x10b604b1
10b60414:	8b 76 34             	mov    esi,DWORD PTR [esi+0x34]
10b60417:	8b 4e 0c             	mov    ecx,DWORD PTR [esi+0xc]
10b6041a:	8b 16                	mov    edx,DWORD PTR [esi]
10b6041c:	3b 51 20             	cmp    edx,DWORD PTR [ecx+0x20]
10b6041f:	0f 85 8c 00 00 00    	jne    0x10b604b1
10b60425:	ff 45 f4             	inc    DWORD PTR [ebp-0xc]
10b60428:	ff 4d f8             	dec    DWORD PTR [ebp-0x8]
10b6042b:	3d f0 02 00 00       	cmp    eax,0x2f0
10b60430:	75 7f                	jne    0x10b604b1
10b60432:	ff 45 f0             	inc    DWORD PTR [ebp-0x10]
10b60435:	eb 7a                	jmp    0x10b604b1
10b60437:	ff 4d f4             	dec    DWORD PTR [ebp-0xc]
10b6043a:	3d f1 02 00 00       	cmp    eax,0x2f1
10b6043f:	75 70                	jne    0x10b604b1
10b60441:	ff 4d f0             	dec    DWORD PTR [ebp-0x10]
10b60444:	eb 6b                	jmp    0x10b604b1
10b60446:	8a 46 08             	mov    al,BYTE PTR [esi+0x8]
10b60449:	3c 17                	cmp    al,0x17
10b6044b:	75 18                	jne    0x10b60465
10b6044d:	81 7e 04 12 03 00 00 	cmp    DWORD PTR [esi+0x4],0x312
10b60454:	75 5b                	jne    0x10b604b1
10b60456:	8b 46 20             	mov    eax,DWORD PTR [esi+0x20]
10b60459:	8b ce                	mov    ecx,esi
10b6045b:	89 45 dc             	mov    DWORD PTR [ebp-0x24],eax
10b6045e:	e8 b3 50 07 00       	call   0x10bd5516
10b60463:	eb 4c                	jmp    0x10b604b1
10b60465:	3c 1b                	cmp    al,0x1b
10b60467:	75 1c                	jne    0x10b60485
10b60469:	8b 46 24             	mov    eax,DWORD PTR [esi+0x24]
10b6046c:	80 78 31 54          	cmp    BYTE PTR [eax+0x31],0x54
10b60470:	75 3f                	jne    0x10b604b1
10b60472:	8b c1                	mov    eax,ecx
10b60474:	b9 01 03 00 00       	mov    ecx,0x301
10b60479:	39 48 04             	cmp    DWORD PTR [eax+0x4],ecx
10b6047c:	8b 00                	mov    eax,DWORD PTR [eax]
10b6047e:	75 f9                	jne    0x10b60479
10b60480:	89 45 cc             	mov    DWORD PTR [ebp-0x34],eax
10b60483:	eb 2c                	jmp    0x10b604b1
10b60485:	3c 1a                	cmp    al,0x1a
10b60487:	75 28                	jne    0x10b604b1
10b60489:	8b 76 20             	mov    esi,DWORD PTR [esi+0x20]
10b6048c:	3b f3                	cmp    esi,ebx
10b6048e:	74 21                	je     0x10b604b1
10b60490:	39 5d 0c             	cmp    DWORD PTR [ebp+0xc],ebx
10b60493:	74 1c                	je     0x10b604b1
10b60495:	8b 86 c8 00 00 00    	mov    eax,DWORD PTR [esi+0xc8]
10b6049b:	89 45 c0             	mov    DWORD PTR [ebp-0x40],eax
10b6049e:	8b 86 cc 00 00 00    	mov    eax,DWORD PTR [esi+0xcc]
10b604a4:	89 45 c4             	mov    DWORD PTR [ebp-0x3c],eax
10b604a7:	0f bf 86 ba 00 00 00 	movsx  eax,WORD PTR [esi+0xba]
10b604ae:	89 45 d0             	mov    DWORD PTR [ebp-0x30],eax
10b604b1:	8b 75 cc             	mov    esi,DWORD PTR [ebp-0x34]
10b604b4:	33 db                	xor    ebx,ebx
10b604b6:	3b f3                	cmp    esi,ebx
10b604b8:	0f 85 76 fc ff ff    	jne    0x10b60134
10b604be:	5f                   	pop    edi
10b604bf:	83 3d c8 f5 c3 10 ff 	cmp    DWORD PTR ds:0x10c3f5c8,0xffffffff
10b604c6:	75 08                	jne    0x10b604d0
10b604c8:	8b 45 d8             	mov    eax,DWORD PTR [ebp-0x28]
10b604cb:	a3 c8 f5 c3 10       	mov    ds:0x10c3f5c8,eax
10b604d0:	8b 55 e8             	mov    edx,DWORD PTR [ebp-0x18]
10b604d3:	3b d3                	cmp    edx,ebx
10b604d5:	74 2c                	je     0x10b60503
10b604d7:	8b 4a 04             	mov    ecx,DWORD PTR [edx+0x4]
10b604da:	e8 cf ba ff ff       	call   0x10b5bfae
10b604df:	f6 40 4c 10          	test   BYTE PTR [eax+0x4c],0x10
10b604e3:	74 18                	je     0x10b604fd
10b604e5:	8b 4d d4             	mov    ecx,DWORD PTR [ebp-0x2c]
10b604e8:	eb 07                	jmp    0x10b604f1
10b604ea:	39 41 04             	cmp    DWORD PTR [ecx+0x4],eax
10b604ed:	74 08                	je     0x10b604f7
10b604ef:	8b 09                	mov    ecx,DWORD PTR [ecx]
10b604f1:	3b cb                	cmp    ecx,ebx
10b604f3:	75 f5                	jne    0x10b604ea
10b604f5:	eb 06                	jmp    0x10b604fd
10b604f7:	8a 41 08             	mov    al,BYTE PTR [ecx+0x8]
10b604fa:	88 42 18             	mov    BYTE PTR [edx+0x18],al
10b604fd:	8b 12                	mov    edx,DWORD PTR [edx]
10b604ff:	3b d3                	cmp    edx,ebx
10b60501:	75 d4                	jne    0x10b604d7
10b60503:	8b 45 e8             	mov    eax,DWORD PTR [ebp-0x18]
10b60506:	5e                   	pop    esi
10b60507:	5b                   	pop    ebx
10b60508:	c9                   	leave
10b60509:	c2 08 00             	ret    0x8
10b6050c:	8b ff                	mov    edi,edi
10b6050e:	0c 04                	or     al,0x4
10b60510:	b6 10                	mov    dh,0x10
10b60512:	25 04 b6 10 37       	and    eax,0x3710b604

10b6242a:	55                   	push   ebp
10b6242b:	8b ec                	mov    ebp,esp
10b6242d:	83 ec 14             	sub    esp,0x14
10b62430:	53                   	push   ebx
10b62431:	56                   	push   esi
10b62432:	8b d9                	mov    ebx,ecx
10b62434:	57                   	push   edi
10b62435:	8b 7b 04             	mov    edi,DWORD PTR [ebx+0x4]
10b62438:	89 55 f8             	mov    DWORD PTR [ebp-0x8],edx
10b6243b:	8b 15 e8 72 c4 10    	mov    edx,DWORD PTR ds:0x10c472e8
10b62441:	8b cf                	mov    ecx,edi
10b62443:	89 55 ec             	mov    DWORD PTR [ebp-0x14],edx
10b62446:	e8 63 9b ff ff       	call   0x10b5bfae
10b6244b:	33 c9                	xor    ecx,ecx
10b6244d:	8b f0                	mov    esi,eax
10b6244f:	89 4d f0             	mov    DWORD PTR [ebp-0x10],ecx
10b62452:	89 4d f4             	mov    DWORD PTR [ebp-0xc],ecx
10b62455:	39 0d 3c ed c2 10    	cmp    DWORD PTR ds:0x10c2ed3c,ecx
10b6245b:	75 0d                	jne    0x10b6246a
10b6245d:	89 15 3c ed c2 10    	mov    DWORD PTR ds:0x10c2ed3c,edx
10b62463:	c7 45 f0 01 00 00 00 	mov    DWORD PTR [ebp-0x10],0x1
10b6246a:	8b 46 08             	mov    eax,DWORD PTR [esi+0x8]
10b6246d:	a3 e8 72 c4 10       	mov    ds:0x10c472e8,eax
10b62472:	8b 40 04             	mov    eax,DWORD PTR [eax+0x4]
10b62475:	a3 f8 e2 c2 10       	mov    ds:0x10c2e2f8,eax
10b6247a:	39 0d 20 de c3 10    	cmp    DWORD PTR ds:0x10c3de20,ecx
10b62480:	75 5f                	jne    0x10b624e1
10b62482:	f6 43 1c 10          	test   BYTE PTR [ebx+0x1c],0x10
10b62486:	75 3e                	jne    0x10b624c6
10b62488:	39 0d 08 e3 c2 10    	cmp    DWORD PTR ds:0x10c2e308,ecx
10b6248e:	74 30                	je     0x10b624c0
10b62490:	8b 86 80 00 00 00    	mov    eax,DWORD PTR [esi+0x80]
10b62496:	3b c1                	cmp    eax,ecx
10b62498:	74 26                	je     0x10b624c0
10b6249a:	8b 48 04             	mov    ecx,DWORD PTR [eax+0x4]
10b6249d:	3b f1                	cmp    esi,ecx
10b6249f:	74 11                	je     0x10b624b2
10b624a1:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b624a8:	75 16                	jne    0x10b624c0
10b624aa:	39 8e 90 00 00 00    	cmp    DWORD PTR [esi+0x90],ecx
10b624b0:	75 0e                	jne    0x10b624c0
10b624b2:	8b 80 b1 00 00 00    	mov    eax,DWORD PTR [eax+0xb1]
10b624b8:	c1 e8 0a             	shr    eax,0xa
10b624bb:	83 e0 01             	and    eax,0x1
10b624be:	eb 02                	jmp    0x10b624c2
10b624c0:	33 c0                	xor    eax,eax
10b624c2:	85 c0                	test   eax,eax
10b624c4:	74 1b                	je     0x10b624e1
10b624c6:	a1 10 e3 c2 10       	mov    eax,ds:0x10c2e310
10b624cb:	89 45 f4             	mov    DWORD PTR [ebp-0xc],eax
10b624ce:	8b 86 80 00 00 00    	mov    eax,DWORD PTR [esi+0x80]
10b624d4:	8b 40 76             	mov    eax,DWORD PTR [eax+0x76]
10b624d7:	25 00 00 80 00       	and    eax,0x800000
10b624dc:	a3 10 e3 c2 10       	mov    ds:0x10c2e310,eax
10b624e1:	8b ce                	mov    ecx,esi
10b624e3:	e8 2d 22 06 00       	call   0x10bc4715
10b624e8:	33 d2                	xor    edx,edx
10b624ea:	89 45 fc             	mov    DWORD PTR [ebp-0x4],eax
10b624ed:	66 39 57 14          	cmp    WORD PTR [edi+0x14],dx
10b624f1:	74 18                	je     0x10b6250b
10b624f3:	0f b7 47 14          	movzx  eax,WORD PTR [edi+0x14]
10b624f7:	8b 4d f8             	mov    ecx,DWORD PTR [ebp-0x8]
10b624fa:	a3 ec e2 c2 10       	mov    ds:0x10c2e2ec,eax
10b624ff:	0f b7 47 14          	movzx  eax,WORD PTR [edi+0x14]
10b62503:	03 41 44             	add    eax,DWORD PTR [ecx+0x44]
10b62506:	a3 e0 e2 c2 10       	mov    ds:0x10c2e2e0,eax
10b6250b:	39 15 20 de c3 10    	cmp    DWORD PTR ds:0x10c3de20,edx
10b62511:	75 4c                	jne    0x10b6255f
10b62513:	f6 43 1c 10          	test   BYTE PTR [ebx+0x1c],0x10
10b62517:	75 3e                	jne    0x10b62557
10b62519:	39 15 08 e3 c2 10    	cmp    DWORD PTR ds:0x10c2e308,edx
10b6251f:	74 30                	je     0x10b62551
10b62521:	8b 86 80 00 00 00    	mov    eax,DWORD PTR [esi+0x80]
10b62527:	3b c2                	cmp    eax,edx
10b62529:	74 26                	je     0x10b62551
10b6252b:	8b 48 04             	mov    ecx,DWORD PTR [eax+0x4]
10b6252e:	3b f1                	cmp    esi,ecx
10b62530:	74 11                	je     0x10b62543
10b62532:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b62539:	75 16                	jne    0x10b62551
10b6253b:	39 8e 90 00 00 00    	cmp    DWORD PTR [esi+0x90],ecx
10b62541:	75 0e                	jne    0x10b62551
10b62543:	8b 80 b1 00 00 00    	mov    eax,DWORD PTR [eax+0xb1]
10b62549:	c1 e8 0a             	shr    eax,0xa
10b6254c:	83 e0 01             	and    eax,0x1
10b6254f:	eb 02                	jmp    0x10b62553
10b62551:	33 c0                	xor    eax,eax
10b62553:	3b c2                	cmp    eax,edx
10b62555:	74 08                	je     0x10b6255f
10b62557:	8b 45 f4             	mov    eax,DWORD PTR [ebp-0xc]
10b6255a:	a3 10 e3 c2 10       	mov    ds:0x10c2e310,eax
10b6255f:	8b 46 08             	mov    eax,DWORD PTR [esi+0x8]
10b62562:	f7 80 d8 0c 00 00 00 	test   DWORD PTR [eax+0xcd8],0x20000
10b62569:	00 02 00 
10b6256c:	74 15                	je     0x10b62583
10b6256e:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b62571:	39 91 88 00 00 00    	cmp    DWORD PTR [ecx+0x88],edx
10b62577:	74 0a                	je     0x10b62583
10b62579:	33 d2                	xor    edx,edx
10b6257b:	42                   	inc    edx
10b6257c:	e8 3a 64 ff ff       	call   0x10b589bb
10b62581:	33 d2                	xor    edx,edx
10b62583:	39 15 c4 62 c4 10    	cmp    DWORD PTR ds:0x10c462c4,edx
10b62589:	74 10                	je     0x10b6259b
10b6258b:	8b 45 ec             	mov    eax,DWORD PTR [ebp-0x14]
10b6258e:	a3 e8 72 c4 10       	mov    ds:0x10c472e8,eax
10b62593:	8b 40 04             	mov    eax,DWORD PTR [eax+0x4]
10b62596:	a3 f8 e2 c2 10       	mov    ds:0x10c2e2f8,eax
10b6259b:	39 55 f0             	cmp    DWORD PTR [ebp-0x10],edx
10b6259e:	74 06                	je     0x10b625a6
10b625a0:	89 15 3c ed c2 10    	mov    DWORD PTR ds:0x10c2ed3c,edx
10b625a6:	f7 46 4c 00 20 00 00 	test   DWORD PTR [esi+0x4c],0x2000
10b625ad:	8b 7d 08             	mov    edi,DWORD PTR [ebp+0x8]
10b625b0:	75 15                	jne    0x10b625c7
10b625b2:	0f b7 46 50          	movzx  eax,WORD PTR [esi+0x50]
10b625b6:	83 f8 28             	cmp    eax,0x28
10b625b9:	76 02                	jbe    0x10b625bd
10b625bb:	29 07                	sub    DWORD PTR [edi],eax
10b625bd:	0f b7 46 50          	movzx  eax,WORD PTR [esi+0x50]
10b625c1:	01 05 cc f5 c3 10    	add    DWORD PTR ds:0x10c3f5cc,eax
10b625c7:	ff 75 18             	push   DWORD PTR [ebp+0x18]
10b625ca:	8b 55 f8             	mov    edx,DWORD PTR [ebp-0x8]
10b625cd:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b625d0:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b625d3:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b625d6:	ff 75 0c             	push   DWORD PTR [ebp+0xc]
10b625d9:	57                   	push   edi
10b625da:	53                   	push   ebx
10b625db:	e8 1c fb ff ff       	call   0x10b620fc
10b625e0:	85 c0                	test   eax,eax
10b625e2:	74 08                	je     0x10b625ec
10b625e4:	8b 45 fc             	mov    eax,DWORD PTR [ebp-0x4]
10b625e7:	e9 82 00 00 00       	jmp    0x10b6266e
10b625ec:	83 3d 08 e3 c2 10 00 	cmp    DWORD PTR ds:0x10c2e308,0x0
10b625f3:	74 30                	je     0x10b62625
10b625f5:	8b 86 80 00 00 00    	mov    eax,DWORD PTR [esi+0x80]
10b625fb:	85 c0                	test   eax,eax
10b625fd:	74 26                	je     0x10b62625
10b625ff:	8b 48 04             	mov    ecx,DWORD PTR [eax+0x4]
10b62602:	3b f1                	cmp    esi,ecx
10b62604:	74 11                	je     0x10b62617
10b62606:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b6260d:	75 16                	jne    0x10b62625
10b6260f:	39 8e 90 00 00 00    	cmp    DWORD PTR [esi+0x90],ecx
10b62615:	75 0e                	jne    0x10b62625
10b62617:	8b 80 b1 00 00 00    	mov    eax,DWORD PTR [eax+0xb1]
10b6261d:	c1 e8 0a             	shr    eax,0xa
10b62620:	83 e0 01             	and    eax,0x1
10b62623:	eb 02                	jmp    0x10b62627
10b62625:	33 c0                	xor    eax,eax
10b62627:	85 c0                	test   eax,eax
10b62629:	74 28                	je     0x10b62653
10b6262b:	8b 43 1c             	mov    eax,DWORD PTR [ebx+0x1c]
10b6262e:	a8 10                	test   al,0x10
10b62630:	75 21                	jne    0x10b62653
10b62632:	ff 75 18             	push   DWORD PTR [ebp+0x18]
10b62635:	8b 55 f8             	mov    edx,DWORD PTR [ebp-0x8]
10b62638:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b6263b:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b6263e:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b62641:	83 c8 10             	or     eax,0x10
10b62644:	ff 75 0c             	push   DWORD PTR [ebp+0xc]
10b62647:	89 43 1c             	mov    DWORD PTR [ebx+0x1c],eax
10b6264a:	57                   	push   edi
10b6264b:	53                   	push   ebx
10b6264c:	e8 ab fa ff ff       	call   0x10b620fc
10b62651:	eb 91                	jmp    0x10b625e4
10b62653:	8b 55 0c             	mov    edx,DWORD PTR [ebp+0xc]
10b62656:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b62659:	e8 44 a7 ff ff       	call   0x10b5cda2
10b6265e:	83 3d 20 de c3 10 02 	cmp    DWORD PTR ds:0x10c3de20,0x2
10b62665:	75 05                	jne    0x10b6266c
10b62667:	e8 8d a4 03 00       	call   0x10b9caf9
10b6266c:	33 c0                	xor    eax,eax
10b6266e:	5f                   	pop    edi
10b6266f:	5e                   	pop    esi
10b62670:	5b                   	pop    ebx
10b62671:	c9                   	leave
10b62672:	c2 14 00             	ret    0x14

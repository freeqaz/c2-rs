10b620fc:	55                   	push   ebp
10b620fd:	8b ec                	mov    ebp,esp
10b620ff:	83 ec 10             	sub    esp,0x10
10b62102:	56                   	push   esi
10b62103:	8b f1                	mov    esi,ecx
10b62105:	8b 46 08             	mov    eax,DWORD PTR [esi+0x8]
10b62108:	8b 00                	mov    eax,DWORD PTR [eax]
10b6210a:	8b 48 1c             	mov    ecx,DWORD PTR [eax+0x1c]
10b6210d:	89 55 fc             	mov    DWORD PTR [ebp-0x4],edx
10b62110:	57                   	push   edi
10b62111:	b2 18                	mov    dl,0x18
10b62113:	e8 9a 22 07 00       	call   0x10bd43b2
10b62118:	8b 7d 08             	mov    edi,DWORD PTR [ebp+0x8]
10b6211b:	8b 4f 04             	mov    ecx,DWORD PTR [edi+0x4]
10b6211e:	89 45 f8             	mov    DWORD PTR [ebp-0x8],eax
10b62121:	89 4d 08             	mov    DWORD PTR [ebp+0x8],ecx
10b62124:	e8 85 9e ff ff       	call   0x10b5bfae
10b62129:	f6 86 94 00 00 00 20 	test   BYTE PTR [esi+0x94],0x20
10b62130:	89 45 f4             	mov    DWORD PTR [ebp-0xc],eax
10b62133:	74 0c                	je     0x10b62141
10b62135:	e8 64 43 06 00       	call   0x10bc649e
10b6213a:	33 c0                	xor    eax,eax
10b6213c:	e9 e3 02 00 00       	jmp    0x10b62424
10b62141:	53                   	push   ebx
10b62142:	8b 5f 1c             	mov    ebx,DWORD PTR [edi+0x1c]
10b62145:	33 c9                	xor    ecx,ecx
10b62147:	41                   	inc    ecx
10b62148:	8b c3                	mov    eax,ebx
10b6214a:	c1 e8 03             	shr    eax,0x3
10b6214d:	23 c1                	and    eax,ecx
10b6214f:	50                   	push   eax
10b62150:	8b c3                	mov    eax,ebx
10b62152:	23 c1                	and    eax,ecx
10b62154:	50                   	push   eax
10b62155:	8b c3                	mov    eax,ebx
10b62157:	c1 e8 02             	shr    eax,0x2
10b6215a:	23 c1                	and    eax,ecx
10b6215c:	50                   	push   eax
10b6215d:	8b 45 fc             	mov    eax,DWORD PTR [ebp-0x4]
10b62160:	8b 90 94 00 00 00    	mov    edx,DWORD PTR [eax+0x94]
10b62166:	8b 00                	mov    eax,DWORD PTR [eax]
10b62168:	c1 ea 02             	shr    edx,0x2
10b6216b:	23 d1                	and    edx,ecx
10b6216d:	52                   	push   edx
10b6216e:	8b 50 20             	mov    edx,DWORD PTR [eax+0x20]
10b62171:	c1 ea 0c             	shr    edx,0xc
10b62174:	23 d1                	and    edx,ecx
10b62176:	8b 4d 18             	mov    ecx,DWORD PTR [ebp+0x18]
10b62179:	e8 dd a7 ff ff       	call   0x10b5c95b
10b6217e:	89 45 18             	mov    DWORD PTR [ebp+0x18],eax
10b62181:	f6 c3 10             	test   bl,0x10
10b62184:	75 2e                	jne    0x10b621b4
10b62186:	8b 86 94 00 00 00    	mov    eax,DWORD PTR [esi+0x94]
10b6218c:	ff 75 18             	push   DWORD PTR [ebp+0x18]
10b6218f:	8b 55 f4             	mov    edx,DWORD PTR [ebp-0xc]
10b62192:	8b c8                	mov    ecx,eax
10b62194:	d1 e9                	shr    ecx,1
10b62196:	83 e1 01             	and    ecx,0x1
10b62199:	51                   	push   ecx
10b6219a:	ff 75 1c             	push   DWORD PTR [ebp+0x1c]
10b6219d:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b621a0:	c1 e8 0a             	shr    eax,0xa
10b621a3:	83 e0 01             	and    eax,0x1
10b621a6:	50                   	push   eax
10b621a7:	e8 23 a8 ff ff       	call   0x10b5c9cf
10b621ac:	85 c0                	test   eax,eax
10b621ae:	0f 84 fb 01 00 00    	je     0x10b623af
10b621b4:	8b 86 98 00 00 00    	mov    eax,DWORD PTR [esi+0x98]
10b621ba:	bb 00 10 00 00       	mov    ebx,0x1000
10b621bf:	85 c3                	test   ebx,eax
10b621c1:	74 12                	je     0x10b621d5
10b621c3:	a9 00 20 00 00       	test   eax,0x2000
10b621c8:	0f 85 e1 01 00 00    	jne    0x10b623af
10b621ce:	8b ce                	mov    ecx,esi
10b621d0:	e8 3f cc 02 00       	call   0x10b8ee14
10b621d5:	8b ce                	mov    ecx,esi
10b621d7:	e8 0a 3a 02 00       	call   0x10b85be6
10b621dc:	f7 86 94 00 00 00 00 	test   DWORD PTR [esi+0x94],0xc00000
10b621e3:	00 c0 00 
10b621e6:	0f 84 0a 01 00 00    	je     0x10b622f6
10b621ec:	8b 45 fc             	mov    eax,DWORD PTR [ebp-0x4]
10b621ef:	8b 00                	mov    eax,DWORD PTR [eax]
10b621f1:	85 58 20             	test   DWORD PTR [eax+0x20],ebx
10b621f4:	74 4f                	je     0x10b62245
10b621f6:	8b 57 1c             	mov    edx,DWORD PTR [edi+0x1c]
10b621f9:	f6 c2 10             	test   dl,0x10
10b621fc:	75 47                	jne    0x10b62245
10b621fe:	83 3d 08 e3 c2 10 00 	cmp    DWORD PTR ds:0x10c2e308,0x0
10b62205:	74 32                	je     0x10b62239
10b62207:	8b 06                	mov    eax,DWORD PTR [esi]
10b62209:	8b 88 80 00 00 00    	mov    ecx,DWORD PTR [eax+0x80]
10b6220f:	85 c9                	test   ecx,ecx
10b62211:	74 26                	je     0x10b62239
10b62213:	8b 59 04             	mov    ebx,DWORD PTR [ecx+0x4]
10b62216:	3b c3                	cmp    eax,ebx
10b62218:	74 11                	je     0x10b6222b
10b6221a:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b62221:	75 16                	jne    0x10b62239
10b62223:	39 98 90 00 00 00    	cmp    DWORD PTR [eax+0x90],ebx
10b62229:	75 0e                	jne    0x10b62239
10b6222b:	8b 81 b1 00 00 00    	mov    eax,DWORD PTR [ecx+0xb1]
10b62231:	c1 e8 0a             	shr    eax,0xa
10b62234:	83 e0 01             	and    eax,0x1
10b62237:	eb 02                	jmp    0x10b6223b
10b62239:	33 c0                	xor    eax,eax
10b6223b:	85 c0                	test   eax,eax
10b6223d:	74 4f                	je     0x10b6228e
10b6223f:	83 ca 10             	or     edx,0x10
10b62242:	89 57 1c             	mov    DWORD PTR [edi+0x1c],edx
10b62245:	8b 47 1c             	mov    eax,DWORD PTR [edi+0x1c]
10b62248:	33 d2                	xor    edx,edx
10b6224a:	c1 e8 04             	shr    eax,0x4
10b6224d:	42                   	inc    edx
10b6224e:	23 c2                	and    eax,edx
10b62250:	50                   	push   eax
10b62251:	8b ce                	mov    ecx,esi
10b62253:	e8 7c 18 08 00       	call   0x10be3ad4
10b62258:	f6 47 1c 10          	test   BYTE PTR [edi+0x1c],0x10
10b6225c:	74 52                	je     0x10b622b0
10b6225e:	a1 a8 34 c4 10       	mov    eax,ds:0x10c434a8
10b62263:	83 25 a8 34 c4 10 00 	and    DWORD PTR ds:0x10c434a8,0x0
10b6226a:	89 45 1c             	mov    DWORD PTR [ebp+0x1c],eax
10b6226d:	a1 d0 34 c4 10       	mov    eax,ds:0x10c434d0
10b62272:	89 45 f0             	mov    DWORD PTR [ebp-0x10],eax
10b62275:	e8 cf 1e 07 00       	call   0x10bd4149
10b6227a:	8b d8                	mov    ebx,eax
10b6227c:	89 1d e4 34 c4 10    	mov    DWORD PTR ds:0x10c434e4,ebx
10b62282:	e8 c2 1e 07 00       	call   0x10bd4149
10b62287:	89 03                	mov    DWORD PTR [ebx],eax
10b62289:	89 58 10             	mov    DWORD PTR [eax+0x10],ebx
10b6228c:	eb 25                	jmp    0x10b622b3
10b6228e:	83 3d 20 de c3 10 02 	cmp    DWORD PTR ds:0x10c3de20,0x2
10b62295:	0f 85 14 01 00 00    	jne    0x10b623af
10b6229b:	8b 55 08             	mov    edx,DWORD PTR [ebp+0x8]
10b6229e:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b622a1:	68 10 26 b0 10       	push   0x10b02610
10b622a6:	e8 eb c4 03 00       	call   0x10b9e796
10b622ab:	e9 ff 00 00 00       	jmp    0x10b623af
10b622b0:	8b 5d 18             	mov    ebx,DWORD PTR [ebp+0x18]
10b622b3:	33 d2                	xor    edx,edx
10b622b5:	42                   	inc    edx
10b622b6:	8b ce                	mov    ecx,esi
10b622b8:	e8 73 12 08 00       	call   0x10be3530
10b622bd:	33 d2                	xor    edx,edx
10b622bf:	42                   	inc    edx
10b622c0:	8b ce                	mov    ecx,esi
10b622c2:	e8 f1 07 08 00       	call   0x10be2ab8
10b622c7:	f6 47 1c 10          	test   BYTE PTR [edi+0x1c],0x10
10b622cb:	74 29                	je     0x10b622f6
10b622cd:	8b 45 1c             	mov    eax,DWORD PTR [ebp+0x1c]
10b622d0:	a3 a8 34 c4 10       	mov    ds:0x10c434a8,eax
10b622d5:	8b 45 f0             	mov    eax,DWORD PTR [ebp-0x10]
10b622d8:	a3 d0 34 c4 10       	mov    ds:0x10c434d0,eax
10b622dd:	eb 07                	jmp    0x10b622e6
10b622df:	8b 19                	mov    ebx,DWORD PTR [ecx]
10b622e1:	e8 42 2f 07 00       	call   0x10bd5228
10b622e6:	8b cb                	mov    ecx,ebx
10b622e8:	85 db                	test   ebx,ebx
10b622ea:	75 f3                	jne    0x10b622df
10b622ec:	a1 e0 34 c4 10       	mov    eax,ds:0x10c434e0
10b622f1:	a3 e4 34 c4 10       	mov    ds:0x10c434e4,eax
10b622f6:	83 3d c8 f1 c6 10 00 	cmp    DWORD PTR ds:0x10c6f1c8,0x0
10b622fd:	74 07                	je     0x10b62306
10b622ff:	8b ce                	mov    ecx,esi
10b62301:	e8 06 c0 01 00       	call   0x10b7e30c
10b62306:	a1 20 de c3 10       	mov    eax,ds:0x10c3de20
10b6230b:	83 f8 02             	cmp    eax,0x2
10b6230e:	75 15                	jne    0x10b62325
10b62310:	8b 55 08             	mov    edx,DWORD PTR [ebp+0x8]
10b62313:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b62316:	56                   	push   esi
10b62317:	e8 11 f9 03 00       	call   0x10ba1c2d
10b6231c:	85 c0                	test   eax,eax
10b6231e:	75 4b                	jne    0x10b6236b
10b62320:	e9 8a 00 00 00       	jmp    0x10b623af
10b62325:	83 f8 01             	cmp    eax,0x1
10b62328:	75 41                	jne    0x10b6236b
10b6232a:	8b 0e                	mov    ecx,DWORD PTR [esi]
10b6232c:	8b 81 80 00 00 00    	mov    eax,DWORD PTR [ecx+0x80]
10b62332:	ff b0 8e 00 00 00    	push   DWORD PTR [eax+0x8e]
10b62338:	33 d2                	xor    edx,edx
10b6233a:	e8 84 7a 03 00       	call   0x10b99dc3
10b6233f:	50                   	push   eax
10b62340:	68 ec 25 b0 10       	push   0x10b025ec
10b62345:	6a 7f                	push   0x7f
10b62347:	e8 8b bb 03 00       	call   0x10b9ded7
10b6234c:	8b 45 fc             	mov    eax,DWORD PTR [ebp-0x4]
10b6234f:	8b 08                	mov    ecx,DWORD PTR [eax]
10b62351:	83 c4 10             	add    esp,0x10
10b62354:	33 d2                	xor    edx,edx
10b62356:	e8 68 7a 03 00       	call   0x10b99dc3
10b6235b:	50                   	push   eax
10b6235c:	68 d8 25 b0 10       	push   0x10b025d8
10b62361:	6a 7f                	push   0x7f
10b62363:	e8 6f bb 03 00       	call   0x10b9ded7
10b62368:	83 c4 0c             	add    esp,0xc
10b6236b:	f6 47 1c 10          	test   BYTE PTR [edi+0x1c],0x10
10b6236f:	74 10                	je     0x10b62381
10b62371:	8b 55 08             	mov    edx,DWORD PTR [ebp+0x8]
10b62374:	8b ce                	mov    ecx,esi
10b62376:	e8 fe a0 05 00       	call   0x10bbc479
10b6237b:	f6 47 1c 10          	test   BYTE PTR [edi+0x1c],0x10
10b6237f:	75 32                	jne    0x10b623b3
10b62381:	33 db                	xor    ebx,ebx
10b62383:	f7 86 94 00 00 00 00 	test   DWORD PTR [esi+0x94],0x500000
10b6238a:	00 50 00 
10b6238d:	74 01                	je     0x10b62390
10b6238f:	43                   	inc    ebx
10b62390:	8b 4d f8             	mov    ecx,DWORD PTR [ebp-0x8]
10b62393:	e8 63 a5 ff ff       	call   0x10b5c8fb
10b62398:	8b 55 08             	mov    edx,DWORD PTR [ebp+0x8]
10b6239b:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b6239e:	f7 d8                	neg    eax
10b623a0:	1b c0                	sbb    eax,eax
10b623a2:	f7 d8                	neg    eax
10b623a4:	50                   	push   eax
10b623a5:	53                   	push   ebx
10b623a6:	e8 4f a6 ff ff       	call   0x10b5c9fa
10b623ab:	85 c0                	test   eax,eax
10b623ad:	75 04                	jne    0x10b623b3
10b623af:	33 c0                	xor    eax,eax
10b623b1:	eb 70                	jmp    0x10b62423
10b623b3:	8b 06                	mov    eax,DWORD PTR [esi]
10b623b5:	8b 58 20             	mov    ebx,DWORD PTR [eax+0x20]
10b623b8:	ff 75 08             	push   DWORD PTR [ebp+0x8]
10b623bb:	8b 55 f8             	mov    edx,DWORD PTR [ebp-0x8]
10b623be:	8b ce                	mov    ecx,esi
10b623c0:	83 e3 1c             	and    ebx,0x1c
10b623c3:	e8 86 b1 ff ff       	call   0x10b5d54e
10b623c8:	85 c0                	test   eax,eax
10b623ca:	7e 1a                	jle    0x10b623e6
10b623cc:	8b 4d f8             	mov    ecx,DWORD PTR [ebp-0x8]
10b623cf:	8b d0                	mov    edx,eax
10b623d1:	e8 59 b2 ff ff       	call   0x10b5d62f
10b623d6:	85 c0                	test   eax,eax
10b623d8:	7e 0c                	jle    0x10b623e6
10b623da:	8b 55 10             	mov    edx,DWORD PTR [ebp+0x10]
10b623dd:	8b 4d f8             	mov    ecx,DWORD PTR [ebp-0x8]
10b623e0:	53                   	push   ebx
10b623e1:	e8 6a b3 ff ff       	call   0x10b5d750
10b623e6:	8b 5d 0c             	mov    ebx,DWORD PTR [ebp+0xc]
10b623e9:	8b 03                	mov    eax,DWORD PTR [ebx]
10b623eb:	99                   	cdq
10b623ec:	f7 7d 14             	idiv   DWORD PTR [ebp+0x14]
10b623ef:	ff 77 14             	push   DWORD PTR [edi+0x14]
10b623f2:	0f b6 57 18          	movzx  edx,BYTE PTR [edi+0x18]
10b623f6:	ff 77 10             	push   DWORD PTR [edi+0x10]
10b623f9:	03 55 10             	add    edx,DWORD PTR [ebp+0x10]
10b623fc:	ff 75 18             	push   DWORD PTR [ebp+0x18]
10b623ff:	8b ce                	mov    ecx,esi
10b62401:	50                   	push   eax
10b62402:	e8 da fa ff ff       	call   0x10b61ee1
10b62407:	8b 4d f4             	mov    ecx,DWORD PTR [ebp-0xc]
10b6240a:	a3 d0 f5 c3 10       	mov    ds:0x10c3f5d0,eax
10b6240f:	f7 41 4c 00 20 00 00 	test   DWORD PTR [ecx+0x4c],0x2000
10b62416:	75 08                	jne    0x10b62420
10b62418:	29 03                	sub    DWORD PTR [ebx],eax
10b6241a:	01 05 cc f5 c3 10    	add    DWORD PTR ds:0x10c3f5cc,eax
10b62420:	33 c0                	xor    eax,eax
10b62422:	40                   	inc    eax
10b62423:	5b                   	pop    ebx
10b62424:	5f                   	pop    edi
10b62425:	5e                   	pop    esi
10b62426:	c9                   	leave
10b62427:	c2 18 00             	ret    0x18

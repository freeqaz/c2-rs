10b5fb5f:	55                   	push   ebp
10b5fb60:	8b ec                	mov    ebp,esp
10b5fb62:	51                   	push   ecx
10b5fb63:	51                   	push   ecx
10b5fb64:	53                   	push   ebx
10b5fb65:	8b d9                	mov    ebx,ecx
10b5fb67:	56                   	push   esi
10b5fb68:	89 55 fc             	mov    DWORD PTR [ebp-0x4],edx
10b5fb6b:	89 5d f8             	mov    DWORD PTR [ebp-0x8],ebx
10b5fb6e:	e8 3b c4 ff ff       	call   0x10b5bfae
10b5fb73:	f6 43 34 01          	test   BYTE PTR [ebx+0x34],0x1
10b5fb77:	8b f0                	mov    esi,eax
10b5fb79:	74 07                	je     0x10b5fb82
10b5fb7b:	33 c0                	xor    eax,eax
10b5fb7d:	e9 50 01 00 00       	jmp    0x10b5fcd2
10b5fb82:	57                   	push   edi
10b5fb83:	33 ff                	xor    edi,edi
10b5fb85:	39 3d 08 e3 c2 10    	cmp    DWORD PTR ds:0x10c2e308,edi
10b5fb8b:	74 30                	je     0x10b5fbbd
10b5fb8d:	8b 86 80 00 00 00    	mov    eax,DWORD PTR [esi+0x80]
10b5fb93:	3b c7                	cmp    eax,edi
10b5fb95:	74 26                	je     0x10b5fbbd
10b5fb97:	8b 48 04             	mov    ecx,DWORD PTR [eax+0x4]
10b5fb9a:	3b f1                	cmp    esi,ecx
10b5fb9c:	74 11                	je     0x10b5fbaf
10b5fb9e:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b5fba5:	75 16                	jne    0x10b5fbbd
10b5fba7:	39 8e 90 00 00 00    	cmp    DWORD PTR [esi+0x90],ecx
10b5fbad:	75 0e                	jne    0x10b5fbbd
10b5fbaf:	8b 80 b1 00 00 00    	mov    eax,DWORD PTR [eax+0xb1]
10b5fbb5:	c1 e8 0a             	shr    eax,0xa
10b5fbb8:	83 e0 01             	and    eax,0x1
10b5fbbb:	eb 02                	jmp    0x10b5fbbf
10b5fbbd:	33 c0                	xor    eax,eax
10b5fbbf:	3b c7                	cmp    eax,edi
10b5fbc1:	74 08                	je     0x10b5fbcb
10b5fbc3:	39 3d b0 ea c2 10    	cmp    DWORD PTR ds:0x10c2eab0,edi
10b5fbc9:	74 13                	je     0x10b5fbde
10b5fbcb:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b5fbce:	8b cb                	mov    ecx,ebx
10b5fbd0:	56                   	push   esi
10b5fbd1:	e8 5b e9 ff ff       	call   0x10b5e531
10b5fbd6:	85 c0                	test   eax,eax
10b5fbd8:	0f 84 ec 00 00 00    	je     0x10b5fcca
10b5fbde:	83 3d 20 de c3 10 01 	cmp    DWORD PTR ds:0x10c3de20,0x1
10b5fbe5:	75 12                	jne    0x10b5fbf9
10b5fbe7:	f7 86 94 00 00 00 00 	test   DWORD PTR [esi+0x94],0x400
10b5fbee:	04 00 00 
10b5fbf1:	74 06                	je     0x10b5fbf9
10b5fbf3:	8b b6 90 00 00 00    	mov    esi,DWORD PTR [esi+0x90]
10b5fbf9:	8b 5e 4c             	mov    ebx,DWORD PTR [esi+0x4c]
10b5fbfc:	f6 c3 10             	test   bl,0x10
10b5fbff:	74 0d                	je     0x10b5fc0e
10b5fc01:	f7 45 0c 00 0f 00 00 	test   DWORD PTR [ebp+0xc],0xf00
10b5fc08:	0f 84 bc 00 00 00    	je     0x10b5fcca
10b5fc0e:	8b 55 fc             	mov    edx,DWORD PTR [ebp-0x4]
10b5fc11:	8b ce                	mov    ecx,esi
10b5fc13:	e8 53 c4 ff ff       	call   0x10b5c06b
10b5fc18:	85 c0                	test   eax,eax
10b5fc1a:	0f 84 aa 00 00 00    	je     0x10b5fcca
10b5fc20:	39 7d 08             	cmp    DWORD PTR [ebp+0x8],edi
10b5fc23:	74 0c                	je     0x10b5fc31
10b5fc25:	f7 c3 00 01 00 00    	test   ebx,0x100
10b5fc2b:	0f 85 99 00 00 00    	jne    0x10b5fcca
10b5fc31:	bf 00 20 00 00       	mov    edi,0x2000
10b5fc36:	85 df                	test   edi,ebx
10b5fc38:	75 2d                	jne    0x10b5fc67
10b5fc3a:	f7 c3 00 02 00 00    	test   ebx,0x200
10b5fc40:	74 25                	je     0x10b5fc67
10b5fc42:	33 db                	xor    ebx,ebx
10b5fc44:	39 1d fc e2 c2 10    	cmp    DWORD PTR ds:0x10c2e2fc,ebx
10b5fc4a:	74 1d                	je     0x10b5fc69
10b5fc4c:	83 3d 20 de c3 10 02 	cmp    DWORD PTR ds:0x10c3de20,0x2
10b5fc53:	75 75                	jne    0x10b5fcca
10b5fc55:	8b 55 f8             	mov    edx,DWORD PTR [ebp-0x8]
10b5fc58:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b5fc5b:	68 88 25 b0 10       	push   0x10b02588
10b5fc60:	e8 31 eb 03 00       	call   0x10b9e796
10b5fc65:	eb 63                	jmp    0x10b5fcca
10b5fc67:	33 db                	xor    ebx,ebx
10b5fc69:	83 3d 20 de c3 10 02 	cmp    DWORD PTR ds:0x10c3de20,0x2
10b5fc70:	75 0c                	jne    0x10b5fc7e
10b5fc72:	8b 4d f8             	mov    ecx,DWORD PTR [ebp-0x8]
10b5fc75:	e8 6c ce 03 00       	call   0x10b9cae6
10b5fc7a:	85 c0                	test   eax,eax
10b5fc7c:	74 50                	je     0x10b5fcce
10b5fc7e:	39 1d 10 e3 c2 10    	cmp    DWORD PTR ds:0x10c2e310,ebx
10b5fc84:	75 33                	jne    0x10b5fcb9
10b5fc86:	0f b7 46 50          	movzx  eax,WORD PTR [esi+0x50]
10b5fc8a:	3b 05 18 63 c4 10    	cmp    eax,DWORD PTR ds:0x10c46318
10b5fc90:	7c 27                	jl     0x10b5fcb9
10b5fc92:	8b 46 4c             	mov    eax,DWORD PTR [esi+0x4c]
10b5fc95:	85 c7                	test   edi,eax
10b5fc97:	75 20                	jne    0x10b5fcb9
10b5fc99:	39 1d fc e2 c2 10    	cmp    DWORD PTR ds:0x10c2e2fc,ebx
10b5fc9f:	74 29                	je     0x10b5fcca
10b5fca1:	39 9e 80 00 00 00    	cmp    DWORD PTR [esi+0x80],ebx
10b5fca7:	74 21                	je     0x10b5fcca
10b5fca9:	a8 02                	test   al,0x2
10b5fcab:	75 08                	jne    0x10b5fcb5
10b5fcad:	39 1d ac ea c2 10    	cmp    DWORD PTR ds:0x10c2eaac,ebx
10b5fcb3:	74 15                	je     0x10b5fcca
10b5fcb5:	a8 10                	test   al,0x10
10b5fcb7:	75 11                	jne    0x10b5fcca
10b5fcb9:	39 1d fc e2 c2 10    	cmp    DWORD PTR ds:0x10c2e2fc,ebx
10b5fcbf:	75 0d                	jne    0x10b5fcce
10b5fcc1:	f7 46 4c 80 20 00 00 	test   DWORD PTR [esi+0x4c],0x2080
10b5fcc8:	75 04                	jne    0x10b5fcce
10b5fcca:	33 c0                	xor    eax,eax
10b5fccc:	eb 03                	jmp    0x10b5fcd1
10b5fcce:	33 c0                	xor    eax,eax
10b5fcd0:	40                   	inc    eax
10b5fcd1:	5f                   	pop    edi
10b5fcd2:	5e                   	pop    esi
10b5fcd3:	5b                   	pop    ebx
10b5fcd4:	c9                   	leave
10b5fcd5:	c2 0c 00             	ret    0xc
10b5fcd8:	55                   	push   ebp

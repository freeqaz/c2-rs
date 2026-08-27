10b61d2c:	55                   	push   ebp
10b61d2d:	8b ec                	mov    ebp,esp
10b61d2f:	51                   	push   ecx
10b61d30:	51                   	push   ecx
10b61d31:	53                   	push   ebx
10b61d32:	8b d9                	mov    ebx,ecx
10b61d34:	8b 4b 04             	mov    ecx,DWORD PTR [ebx+0x4]
10b61d37:	66 83 79 14 00       	cmp    WORD PTR [ecx+0x14],0x0
10b61d3c:	56                   	push   esi
10b61d3d:	57                   	push   edi
10b61d3e:	8b f2                	mov    esi,edx
10b61d40:	89 4d fc             	mov    DWORD PTR [ebp-0x4],ecx
10b61d43:	74 15                	je     0x10b61d5a
10b61d45:	0f b7 41 14          	movzx  eax,WORD PTR [ecx+0x14]
10b61d49:	a3 ec e2 c2 10       	mov    ds:0x10c2e2ec,eax
10b61d4e:	0f b7 41 14          	movzx  eax,WORD PTR [ecx+0x14]
10b61d52:	03 46 44             	add    eax,DWORD PTR [esi+0x44]
10b61d55:	a3 e0 e2 c2 10       	mov    ds:0x10c2e2e0,eax
10b61d5a:	e8 4f a2 ff ff       	call   0x10b5bfae
10b61d5f:	8b 0d e8 72 c4 10    	mov    ecx,DWORD PTR ds:0x10c472e8
10b61d65:	8b f8                	mov    edi,eax
10b61d67:	8b d7                	mov    edx,edi
10b61d69:	e8 16 a2 ff ff       	call   0x10b5bf84
10b61d6e:	85 c0                	test   eax,eax
10b61d70:	74 25                	je     0x10b61d97
10b61d72:	ff 75 24             	push   DWORD PTR [ebp+0x24]
10b61d75:	8b 45 14             	mov    eax,DWORD PTR [ebp+0x14]
10b61d78:	ff 75 20             	push   DWORD PTR [ebp+0x20]
10b61d7b:	0f b6 00             	movzx  eax,BYTE PTR [eax]
10b61d7e:	50                   	push   eax
10b61d7f:	8b 45 10             	mov    eax,DWORD PTR [ebp+0x10]
10b61d82:	ff 30                	push   DWORD PTR [eax]
10b61d84:	8b d6                	mov    edx,esi
10b61d86:	ff 75 0c             	push   DWORD PTR [ebp+0xc]
10b61d89:	8b cb                	mov    ecx,ebx
10b61d8b:	ff 75 08             	push   DWORD PTR [ebp+0x8]
10b61d8e:	e8 9d eb ff ff       	call   0x10b60930
10b61d93:	85 c0                	test   eax,eax
10b61d95:	75 47                	jne    0x10b61dde
10b61d97:	83 3d 08 e3 c2 10 00 	cmp    DWORD PTR ds:0x10c2e308,0x0
10b61d9e:	74 30                	je     0x10b61dd0
10b61da0:	8b 87 80 00 00 00    	mov    eax,DWORD PTR [edi+0x80]
10b61da6:	85 c0                	test   eax,eax
10b61da8:	74 26                	je     0x10b61dd0
10b61daa:	8b 48 04             	mov    ecx,DWORD PTR [eax+0x4]
10b61dad:	3b f9                	cmp    edi,ecx
10b61daf:	74 11                	je     0x10b61dc2
10b61db1:	83 3d c8 f1 c6 10 01 	cmp    DWORD PTR ds:0x10c6f1c8,0x1
10b61db8:	75 16                	jne    0x10b61dd0
10b61dba:	39 8f 90 00 00 00    	cmp    DWORD PTR [edi+0x90],ecx
10b61dc0:	75 0e                	jne    0x10b61dd0
10b61dc2:	8b 80 b1 00 00 00    	mov    eax,DWORD PTR [eax+0xb1]
10b61dc8:	c1 e8 0a             	shr    eax,0xa
10b61dcb:	83 e0 01             	and    eax,0x1
10b61dce:	eb 02                	jmp    0x10b61dd2
10b61dd0:	33 c0                	xor    eax,eax
10b61dd2:	85 c0                	test   eax,eax
10b61dd4:	0f 84 fe 00 00 00    	je     0x10b61ed8
10b61dda:	83 4b 1c 10          	or     DWORD PTR [ebx+0x1c],0x10
10b61dde:	8b 4d fc             	mov    ecx,DWORD PTR [ebp-0x4]
10b61de1:	8d 45 14             	lea    eax,[ebp+0x14]
10b61de4:	50                   	push   eax
10b61de5:	8b d6                	mov    edx,esi
10b61de7:	e8 91 aa ff ff       	call   0x10b5c87d
10b61dec:	89 45 24             	mov    DWORD PTR [ebp+0x24],eax
10b61def:	85 c0                	test   eax,eax
10b61df1:	0f 84 e1 00 00 00    	je     0x10b61ed8
10b61df7:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b61dfa:	a1 1c e3 c2 10       	mov    eax,ds:0x10c2e31c
10b61dff:	ff 75 18             	push   DWORD PTR [ebp+0x18]
10b61e02:	8b d6                	mov    edx,esi
10b61e04:	ff 75 1c             	push   DWORD PTR [ebp+0x1c]
10b61e07:	8b cb                	mov    ecx,ebx
10b61e09:	ff 75 08             	push   DWORD PTR [ebp+0x8]
10b61e0c:	89 45 f8             	mov    DWORD PTR [ebp-0x8],eax
10b61e0f:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b61e12:	e8 13 06 00 00       	call   0x10b6242a
10b61e17:	8b f8                	mov    edi,eax
10b61e19:	85 ff                	test   edi,edi
10b61e1b:	0f 84 b7 00 00 00    	je     0x10b61ed8
10b61e21:	8b 47 08             	mov    eax,DWORD PTR [edi+0x8]
10b61e24:	8b 00                	mov    eax,DWORD PTR [eax]
10b61e26:	8b 48 1c             	mov    ecx,DWORD PTR [eax+0x1c]
10b61e29:	b2 18                	mov    dl,0x18
10b61e2b:	e8 82 25 07 00       	call   0x10bd43b2
10b61e30:	89 45 0c             	mov    DWORD PTR [ebp+0xc],eax
10b61e33:	8b 43 1c             	mov    eax,DWORD PTR [ebx+0x1c]
10b61e36:	8b c8                	mov    ecx,eax
10b61e38:	c1 e9 03             	shr    ecx,0x3
10b61e3b:	33 db                	xor    ebx,ebx
10b61e3d:	43                   	inc    ebx
10b61e3e:	23 cb                	and    ecx,ebx
10b61e40:	51                   	push   ecx
10b61e41:	8b c8                	mov    ecx,eax
10b61e43:	c1 e8 02             	shr    eax,0x2
10b61e46:	23 c3                	and    eax,ebx
10b61e48:	23 cb                	and    ecx,ebx
10b61e4a:	51                   	push   ecx
10b61e4b:	8b 4d 18             	mov    ecx,DWORD PTR [ebp+0x18]
10b61e4e:	50                   	push   eax
10b61e4f:	8b 86 94 00 00 00    	mov    eax,DWORD PTR [esi+0x94]
10b61e55:	c1 e8 02             	shr    eax,0x2
10b61e58:	23 c3                	and    eax,ebx
10b61e5a:	50                   	push   eax
10b61e5b:	8b 06                	mov    eax,DWORD PTR [esi]
10b61e5d:	8b 50 20             	mov    edx,DWORD PTR [eax+0x20]
10b61e60:	c1 ea 0c             	shr    edx,0xc
10b61e63:	23 d3                	and    edx,ebx
10b61e65:	e8 f1 aa ff ff       	call   0x10b5c95b
10b61e6a:	50                   	push   eax
10b61e6b:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b61e6e:	8b d6                	mov    edx,esi
10b61e70:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b61e73:	8b cf                	mov    ecx,edi
10b61e75:	ff 75 fc             	push   DWORD PTR [ebp-0x4]
10b61e78:	89 45 18             	mov    DWORD PTR [ebp+0x18],eax
10b61e7b:	ff 75 0c             	push   DWORD PTR [ebp+0xc]
10b61e7e:	e8 22 cb ff ff       	call   0x10b5e9a5
10b61e83:	85 c0                	test   eax,eax
10b61e85:	74 20                	je     0x10b61ea7
10b61e87:	ff 75 18             	push   DWORD PTR [ebp+0x18]
10b61e8a:	8b d6                	mov    edx,esi
10b61e8c:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b61e8f:	8b cf                	mov    ecx,edi
10b61e91:	ff 75 fc             	push   DWORD PTR [ebp-0x4]
10b61e94:	ff 75 24             	push   DWORD PTR [ebp+0x24]
10b61e97:	ff 75 0c             	push   DWORD PTR [ebp+0xc]
10b61e9a:	e8 ad f8 ff ff       	call   0x10b6174c
10b61e9f:	85 c0                	test   eax,eax
10b61ea1:	74 04                	je     0x10b61ea7
10b61ea3:	8b c3                	mov    eax,ebx
10b61ea5:	eb 33                	jmp    0x10b61eda
10b61ea7:	8b 55 08             	mov    edx,DWORD PTR [ebp+0x8]
10b61eaa:	8b cf                	mov    ecx,edi
10b61eac:	e8 f1 ae ff ff       	call   0x10b5cda2
10b61eb1:	a1 d0 34 c4 10       	mov    eax,ds:0x10c434d0
10b61eb6:	85 c0                	test   eax,eax
10b61eb8:	74 1e                	je     0x10b61ed8
10b61eba:	8b 00                	mov    eax,DWORD PTR [eax]
10b61ebc:	8b 48 18             	mov    ecx,DWORD PTR [eax+0x18]
10b61ebf:	8b 55 f8             	mov    edx,DWORD PTR [ebp-0x8]
10b61ec2:	3b ca                	cmp    ecx,edx
10b61ec4:	73 08                	jae    0x10b61ece
10b61ec6:	89 15 1c e3 c2 10    	mov    DWORD PTR ds:0x10c2e31c,edx
10b61ecc:	eb 0a                	jmp    0x10b61ed8
10b61ece:	8b 40 1c             	mov    eax,DWORD PTR [eax+0x1c]
10b61ed1:	03 c1                	add    eax,ecx
10b61ed3:	a3 1c e3 c2 10       	mov    ds:0x10c2e31c,eax
10b61ed8:	33 c0                	xor    eax,eax
10b61eda:	5f                   	pop    edi
10b61edb:	5e                   	pop    esi
10b61edc:	5b                   	pop    ebx
10b61edd:	c9                   	leave
10b61ede:	c2 20 00             	ret    0x20

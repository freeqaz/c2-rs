10b61ee1:	55                   	push   ebp
10b61ee2:	8b ec                	mov    ebp,esp
10b61ee4:	83 ec 30             	sub    esp,0x30
10b61ee7:	8b 45 08             	mov    eax,DWORD PTR [ebp+0x8]
10b61eea:	53                   	push   ebx
10b61eeb:	56                   	push   esi
10b61eec:	89 45 d8             	mov    DWORD PTR [ebp-0x28],eax
10b61eef:	33 c0                	xor    eax,eax
10b61ef1:	57                   	push   edi
10b61ef2:	8b f2                	mov    esi,edx
10b61ef4:	8b f9                	mov    edi,ecx
10b61ef6:	89 75 e0             	mov    DWORD PTR [ebp-0x20],esi
10b61ef9:	89 7d f0             	mov    DWORD PTR [ebp-0x10],edi
10b61efc:	c6 45 ff 00          	mov    BYTE PTR [ebp-0x1],0x0
10b61f00:	89 45 e4             	mov    DWORD PTR [ebp-0x1c],eax
10b61f03:	39 05 fc e2 c2 10    	cmp    DWORD PTR ds:0x10c2e2fc,eax
10b61f09:	74 41                	je     0x10b61f4c
10b61f0b:	33 db                	xor    ebx,ebx
10b61f0d:	43                   	inc    ebx
10b61f0e:	39 1d 20 de c3 10    	cmp    DWORD PTR ds:0x10c3de20,ebx
10b61f14:	74 36                	je     0x10b61f4c
10b61f16:	e8 4a 59 fd ff       	call   0x10b37865
10b61f1b:	8b cf                	mov    ecx,edi
10b61f1d:	e8 1f 48 fd ff       	call   0x10b36741
10b61f22:	8b cf                	mov    ecx,edi
10b61f24:	e8 98 44 fd ff       	call   0x10b363c1
10b61f29:	8b cf                	mov    ecx,edi
10b61f2b:	e8 e9 55 fd ff       	call   0x10b37519
10b61f30:	33 d2                	xor    edx,edx
10b61f32:	8b cf                	mov    ecx,edi
10b61f34:	e8 c6 da 00 00       	call   0x10b6f9ff
10b61f39:	8b cf                	mov    ecx,edi
10b61f3b:	e8 01 48 fd ff       	call   0x10b36741
10b61f40:	8b cf                	mov    ecx,edi
10b61f42:	e8 c4 b0 fd ff       	call   0x10b3d00b
10b61f47:	89 5d f8             	mov    DWORD PTR [ebp-0x8],ebx
10b61f4a:	eb 03                	jmp    0x10b61f4f
10b61f4c:	89 45 f8             	mov    DWORD PTR [ebp-0x8],eax
10b61f4f:	8b 0f                	mov    ecx,DWORD PTR [edi]
10b61f51:	8b 41 4c             	mov    eax,DWORD PTR [ecx+0x4c]
10b61f54:	8b d8                	mov    ebx,eax
10b61f56:	83 c8 10             	or     eax,0x10
10b61f59:	83 e3 10             	and    ebx,0x10
10b61f5c:	89 41 4c             	mov    DWORD PTR [ecx+0x4c],eax
10b61f5f:	33 c0                	xor    eax,eax
10b61f61:	89 5d dc             	mov    DWORD PTR [ebp-0x24],ebx
10b61f64:	3b d8                	cmp    ebx,eax
10b61f66:	74 15                	je     0x10b61f7d
10b61f68:	39 05 0c f5 c3 10    	cmp    DWORD PTR ds:0x10c3f50c,eax
10b61f6e:	75 0d                	jne    0x10b61f7d
10b61f70:	c7 45 e4 01 00 00 00 	mov    DWORD PTR [ebp-0x1c],0x1
10b61f77:	89 35 0c f5 c3 10    	mov    DWORD PTR ds:0x10c3f50c,esi
10b61f7d:	f6 45 0c 01          	test   BYTE PTR [ebp+0xc],0x1
10b61f81:	a3 08 f5 c3 10       	mov    ds:0x10c3f508,eax
10b61f86:	75 0b                	jne    0x10b61f93
10b61f88:	8b 0f                	mov    ecx,DWORD PTR [edi]
10b61f8a:	f7 41 20 00 10 00 00 	test   DWORD PTR [ecx+0x20],0x1000
10b61f91:	74 03                	je     0x10b61f96
10b61f93:	33 c0                	xor    eax,eax
10b61f95:	40                   	inc    eax
10b61f96:	ff 75 f8             	push   DWORD PTR [ebp-0x8]
10b61f99:	8d 55 f4             	lea    edx,[ebp-0xc]
10b61f9c:	50                   	push   eax
10b61f9d:	8b cf                	mov    ecx,edi
10b61f9f:	e8 42 e1 ff ff       	call   0x10b600e6
10b61fa4:	8b f0                	mov    esi,eax
10b61fa6:	85 f6                	test   esi,esi
10b61fa8:	0f 84 28 01 00 00    	je     0x10b620d6
10b61fae:	8b 4e 04             	mov    ecx,DWORD PTR [esi+0x4]
10b61fb1:	e8 f8 9f ff ff       	call   0x10b5bfae
10b61fb6:	8b d8                	mov    ebx,eax
10b61fb8:	a1 34 63 c4 10       	mov    eax,ds:0x10c46334
10b61fbd:	89 45 e8             	mov    DWORD PTR [ebp-0x18],eax
10b61fc0:	a1 30 63 c4 10       	mov    eax,ds:0x10c46330
10b61fc5:	33 c9                	xor    ecx,ecx
10b61fc7:	89 45 ec             	mov    DWORD PTR [ebp-0x14],eax
10b61fca:	89 0d 34 63 c4 10    	mov    DWORD PTR ds:0x10c46334,ecx
10b61fd0:	8b 46 04             	mov    eax,DWORD PTR [esi+0x4]
10b61fd3:	0f b7 40 14          	movzx  eax,WORD PTR [eax+0x14]
10b61fd7:	c7 05 30 63 c4 10 34 	mov    DWORD PTR ds:0x10c46330,0x10c46334
10b61fde:	63 c4 10 
10b61fe1:	66 3b c1             	cmp    ax,cx
10b61fe4:	74 14                	je     0x10b61ffa
10b61fe6:	a3 ec e2 c2 10       	mov    ds:0x10c2e2ec,eax
10b61feb:	8b 46 04             	mov    eax,DWORD PTR [esi+0x4]
10b61fee:	0f b7 40 14          	movzx  eax,WORD PTR [eax+0x14]
10b61ff2:	03 47 44             	add    eax,DWORD PTR [edi+0x44]
10b61ff5:	a3 e0 e2 c2 10       	mov    ds:0x10c2e2e0,eax
10b61ffa:	39 4d f8             	cmp    DWORD PTR [ebp-0x8],ecx
10b61ffd:	74 5e                	je     0x10b6205d
10b61fff:	39 4d 14             	cmp    DWORD PTR [ebp+0x14],ecx
10b62002:	77 11                	ja     0x10b62015
10b62004:	72 09                	jb     0x10b6200f
10b62006:	81 7d 10 00 e1 f5 05 	cmp    DWORD PTR [ebp+0x10],0x5f5e100
10b6200d:	77 06                	ja     0x10b62015
10b6200f:	f6 43 4c 10          	test   BYTE PTR [ebx+0x4c],0x10
10b62013:	74 48                	je     0x10b6205d
10b62015:	8b 4e 14             	mov    ecx,DWORD PTR [esi+0x14]
10b62018:	8b 46 10             	mov    eax,DWORD PTR [esi+0x10]
10b6201b:	6a 00                	push   0x0
10b6201d:	68 00 e1 f5 05       	push   0x5f5e100
10b62022:	51                   	push   ecx
10b62023:	50                   	push   eax
10b62024:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b62027:	89 45 d0             	mov    DWORD PTR [ebp-0x30],eax
10b6202a:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b6202d:	8b f9                	mov    edi,ecx
10b6202f:	e8 78 9e ff ff       	call   0x10b5beac
10b62034:	89 46 10             	mov    DWORD PTR [esi+0x10],eax
10b62037:	89 56 14             	mov    DWORD PTR [esi+0x14],edx
10b6203a:	f6 43 4c 10          	test   BYTE PTR [ebx+0x4c],0x10
10b6203e:	74 1a                	je     0x10b6205a
10b62040:	3b fa                	cmp    edi,edx
10b62042:	77 16                	ja     0x10b6205a
10b62044:	8b 45 d0             	mov    eax,DWORD PTR [ebp-0x30]
10b62047:	72 05                	jb     0x10b6204e
10b62049:	3b 46 10             	cmp    eax,DWORD PTR [esi+0x10]
10b6204c:	77 0c                	ja     0x10b6205a
10b6204e:	0f ac f8 01          	shrd   eax,edi,0x1
10b62052:	d1 ef                	shr    edi,1
10b62054:	89 46 10             	mov    DWORD PTR [esi+0x10],eax
10b62057:	89 7e 14             	mov    DWORD PTR [esi+0x14],edi
10b6205a:	8b 7d f0             	mov    edi,DWORD PTR [ebp-0x10]
10b6205d:	ff 75 14             	push   DWORD PTR [ebp+0x14]
10b62060:	8d 45 ff             	lea    eax,[ebp-0x1]
10b62063:	ff 75 10             	push   DWORD PTR [ebp+0x10]
10b62066:	8b d7                	mov    edx,edi
10b62068:	ff 75 f4             	push   DWORD PTR [ebp-0xc]
10b6206b:	8b ce                	mov    ecx,esi
10b6206d:	ff 75 0c             	push   DWORD PTR [ebp+0xc]
10b62070:	50                   	push   eax
10b62071:	8d 45 08             	lea    eax,[ebp+0x8]
10b62074:	50                   	push   eax
10b62075:	8b 46 08             	mov    eax,DWORD PTR [esi+0x8]
10b62078:	25 ff 00 00 00       	and    eax,0xff
10b6207d:	50                   	push   eax
10b6207e:	ff 75 e0             	push   DWORD PTR [ebp-0x20]
10b62081:	e8 a6 fc ff ff       	call   0x10b61d2c
10b62086:	8b 55 ec             	mov    edx,DWORD PTR [ebp-0x14]
10b62089:	8b 4d e8             	mov    ecx,DWORD PTR [ebp-0x18]
10b6208c:	85 c0                	test   eax,eax
10b6208e:	74 27                	je     0x10b620b7
10b62090:	80 8b 87 00 00 00 02 	or     BYTE PTR [ebx+0x87],0x2
10b62097:	e8 32 96 ff ff       	call   0x10b5b6ce
10b6209c:	8b cb                	mov    ecx,ebx
10b6209e:	e8 98 9f ff ff       	call   0x10b5c03b
10b620a3:	8b cb                	mov    ecx,ebx
10b620a5:	e8 8b 25 07 00       	call   0x10bd4635
10b620aa:	85 c0                	test   eax,eax
10b620ac:	75 18                	jne    0x10b620c6
10b620ae:	8b cb                	mov    ecx,ebx
10b620b0:	e8 25 bf ff ff       	call   0x10b5dfda
10b620b5:	eb 0f                	jmp    0x10b620c6
10b620b7:	e8 e3 95 ff ff       	call   0x10b5b69f
10b620bc:	8b 56 04             	mov    edx,DWORD PTR [esi+0x4]
10b620bf:	8b cb                	mov    ecx,ebx
10b620c1:	e8 fa 9e ff ff       	call   0x10b5bfc0
10b620c6:	8b 36                	mov    esi,DWORD PTR [esi]
10b620c8:	ff 4d f4             	dec    DWORD PTR [ebp-0xc]
10b620cb:	85 f6                	test   esi,esi
10b620cd:	0f 85 db fe ff ff    	jne    0x10b61fae
10b620d3:	8b 5d dc             	mov    ebx,DWORD PTR [ebp-0x24]
10b620d6:	85 db                	test   ebx,ebx
10b620d8:	75 08                	jne    0x10b620e2
10b620da:	8b 3f                	mov    edi,DWORD PTR [edi]
10b620dc:	83 67 4c ef          	and    DWORD PTR [edi+0x4c],0xffffffef
10b620e0:	eb 0d                	jmp    0x10b620ef
10b620e2:	83 7d e4 00          	cmp    DWORD PTR [ebp-0x1c],0x0
10b620e6:	74 07                	je     0x10b620ef
10b620e8:	83 25 0c f5 c3 10 00 	and    DWORD PTR ds:0x10c3f50c,0x0
10b620ef:	8b 45 d8             	mov    eax,DWORD PTR [ebp-0x28]
10b620f2:	2b 45 08             	sub    eax,DWORD PTR [ebp+0x8]
10b620f5:	5f                   	pop    edi
10b620f6:	5e                   	pop    esi
10b620f7:	5b                   	pop    ebx
10b620f8:	c9                   	leave
10b620f9:	c2 10 00             	ret    0x10

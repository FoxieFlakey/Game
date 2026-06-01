let SessionLoad = 1
if &cp | set nocp | endif
let s:cpo_save=&cpo
set cpo&vim
inoremap <silent> <expr> <C-Space> :call CocActionAsync("codeAction")
inoremap <silent> <expr> <PageUp> coc#pum#visible() ? coc#pum#scroll(0) : "\<PageUp>"
inoremap <silent> <expr> <PageDown> coc#pum#visible() ? coc#pum#scroll(1) : "\<PageDown>"
inoremap <silent> <expr> <C-Y> coc#pum#visible() ? coc#pum#confirm() : coc#inline#visible() ? coc#inline#accept() :"\"
inoremap <silent> <expr> <C-E> coc#pum#visible() ? coc#pum#cancel() : coc#inline#visible() ? coc#inline#cancel() : "\"
inoremap <silent> <expr> <Up> coc#pum#visible() ? coc#pum#prev(0) : coc#inline#visible() ? coc#inline#prev() :"\<Up>"
inoremap <silent> <expr> <Down> coc#pum#visible() ? coc#pum#next(0) : coc#inline#visible() ? coc#inline#next() :"\<Down>"
inoremap <silent> <expr> <C-P> coc#pum#visible() ? coc#pum#prev(1) : coc#inline#visible() ? coc#inline#prev() : "\"
inoremap <silent> <expr> <C-N> coc#pum#visible() ? coc#pum#next(1) : coc#inline#visible() ? coc#inline#next() : "\"
inoremap <expr> <S-Tab> coc#pum#visible() ? coc#pum#prev(1) : "\"
snoremap <silent>  c
nnoremap 	 gt
nnoremap  :tabnew | :Explore
snoremap  "_c
nnoremap <expr>  :tabnew
nnoremap <silent> . <Plug>(coc-codeaction)
omap Q gq
xmap Q gq
nmap Q gq
smap Q gq
xmap gx <Plug>(open-word-under-cursor)
nmap gx <Plug>(open-word-under-cursor)
nnoremap <silent> gr <Plug>(coc-references)
nnoremap <silent> gd :call JumpAll()
nnoremap <silent> k :call CocActionAsync('doHover')
snoremap <silent> <BS> c
snoremap <silent> <Del> c
snoremap <silent> <C-H> c
snoremap <C-R> "_c
nnoremap <silent> <C-Space> <Plug>(coc-codeaction)
nnoremap <silent> <C-.> <Plug>(coc-codeaction)
xnoremap <Plug>(open-word-under-cursor) <ScriptCmd>vim9.Open(getregion(getpos('v'), getpos('.'), { type: mode() })->join())
nnoremap <Plug>(open-word-under-cursor) <ScriptCmd>vim9.Open(GetWordUnderCursor())
onoremap <silent> <Plug>(coc-classobj-a) :call CocAction('selectSymbolRange', v:false, '', ['Interface', 'Struct', 'Class'])
onoremap <silent> <Plug>(coc-classobj-i) :call CocAction('selectSymbolRange', v:true, '', ['Interface', 'Struct', 'Class'])
vnoremap <silent> <Plug>(coc-classobj-a) :call CocAction('selectSymbolRange', v:false, visualmode(), ['Interface', 'Struct', 'Class'])
vnoremap <silent> <Plug>(coc-classobj-i) :call CocAction('selectSymbolRange', v:true, visualmode(), ['Interface', 'Struct', 'Class'])
onoremap <silent> <Plug>(coc-funcobj-a) :call CocAction('selectSymbolRange', v:false, '', ['Method', 'Function'])
onoremap <silent> <Plug>(coc-funcobj-i) :call CocAction('selectSymbolRange', v:true, '', ['Method', 'Function'])
vnoremap <silent> <Plug>(coc-funcobj-a) :call CocAction('selectSymbolRange', v:false, visualmode(), ['Method', 'Function'])
vnoremap <silent> <Plug>(coc-funcobj-i) :call CocAction('selectSymbolRange', v:true, visualmode(), ['Method', 'Function'])
nnoremap <silent> <Plug>(coc-cursors-position) :call CocAction('cursorsSelect', bufnr('%'), 'position', 'n')
nnoremap <silent> <Plug>(coc-cursors-word) :call CocAction('cursorsSelect', bufnr('%'), 'word', 'n')
vnoremap <silent> <Plug>(coc-cursors-range) :call CocAction('cursorsSelect', bufnr('%'), 'range', visualmode())
nnoremap <silent> <Plug>(coc-refactor) :call       CocActionAsync('refactor')
nnoremap <silent> <Plug>(coc-command-repeat) :call       CocAction('repeatCommand')
nnoremap <silent> <Plug>(coc-float-jump) :call       coc#float#jump()
nnoremap <silent> <Plug>(coc-float-hide) :call       coc#float#close_all()
nnoremap <silent> <Plug>(coc-fix-current) :call       CocActionAsync('doQuickfix')
nnoremap <silent> <Plug>(coc-openlink) :call       CocActionAsync('openLink')
nnoremap <silent> <Plug>(coc-references-used) :call       CocActionAsync('jumpUsed')
nnoremap <silent> <Plug>(coc-references) :call       CocActionAsync('jumpReferences')
nnoremap <silent> <Plug>(coc-type-definition) :call       CocActionAsync('jumpTypeDefinition')
nnoremap <silent> <Plug>(coc-implementation) :call       CocActionAsync('jumpImplementation')
nnoremap <silent> <Plug>(coc-declaration) :call       CocActionAsync('jumpDeclaration')
nnoremap <silent> <Plug>(coc-definition) :call       CocActionAsync('jumpDefinition')
nnoremap <silent> <Plug>(coc-diagnostic-prev-error) :call       CocActionAsync('diagnosticPrevious', 'error')
nnoremap <silent> <Plug>(coc-diagnostic-next-error) :call       CocActionAsync('diagnosticNext',     'error')
nnoremap <silent> <Plug>(coc-diagnostic-prev) :call       CocActionAsync('diagnosticPrevious')
nnoremap <silent> <Plug>(coc-diagnostic-next) :call       CocActionAsync('diagnosticNext')
nnoremap <silent> <Plug>(coc-diagnostic-info) :call       CocActionAsync('diagnosticInfo')
nnoremap <silent> <Plug>(coc-format) :call       CocActionAsync('format')
nnoremap <silent> <Plug>(coc-rename) :call       CocActionAsync('rename')
nnoremap <Plug>(coc-codeaction-source) :call       CocActionAsync('codeAction', '', ['source'], v:true)
nnoremap <Plug>(coc-codeaction-refactor) :call       CocActionAsync('codeAction', 'cursor', ['refactor'], v:true)
nnoremap <Plug>(coc-codeaction-cursor) :call       CocActionAsync('codeAction', 'cursor')
nnoremap <Plug>(coc-codeaction-line) :call       CocActionAsync('codeAction', 'currline')
nnoremap <Plug>(coc-codeaction) :call       CocActionAsync('codeAction', '')
vnoremap <Plug>(coc-codeaction-refactor-selected) :call       CocActionAsync('codeAction', visualmode(), ['refactor'], v:true)
vnoremap <silent> <Plug>(coc-codeaction-selected) :call       CocActionAsync('codeAction', visualmode())
vnoremap <silent> <Plug>(coc-format-selected) :call       CocActionAsync('formatSelected', visualmode())
nnoremap <Plug>(coc-codelens-action) :call       CocActionAsync('codeLensAction')
nnoremap <Plug>(coc-range-select) :call       CocActionAsync('rangeSelect',     '', v:true)
vnoremap <silent> <Plug>(coc-range-select-backward) :call       CocActionAsync('rangeSelect',     visualmode(), v:false)
vnoremap <silent> <Plug>(coc-range-select) :call       CocActionAsync('rangeSelect',     visualmode(), v:true)
nnoremap <silent> <F2> <Plug>(coc-rename)
nnoremap <C-Down> 
nnoremap <C-Up> 
nnoremap <C-O> :tabnew | :Explore
nnoremap <expr> <C-T> :tabnew
nnoremap <S-Tab> gT
nnoremap <M-Down> <Down>
nnoremap <M-Up> <Up>
nnoremap <M-Left> <Left>
nnoremap <M-Right> <Left>
inoremap <silent> <expr>  coc#pum#visible() ? coc#pum#cancel() : coc#inline#visible() ? coc#inline#cancel() : "\"
inoremap <silent> <expr> 	 coc#pum#visible() ? coc#pum#next(1) : CheckBackspace() ? "\	" : coc#refresh()
inoremap <silent> <expr>  coc#pum#visible() ? coc#pum#confirm(): "\u\\=coc#on_enter()\"
inoremap <silent> <expr>  coc#pum#visible() ? coc#pum#next(1) : coc#inline#visible() ? coc#inline#next() : "\"
inoremap <silent> <expr>  coc#pum#visible() ? coc#pum#prev(1) : coc#inline#visible() ? coc#inline#prev() : "\"
inoremap <silent> <expr>  coc#pum#visible() ? coc#pum#confirm() : coc#inline#visible() ? coc#inline#accept() :"\"
let &cpo=s:cpo_save
unlet s:cpo_save
set autoindent
set autoread
set background=dark
set backspace=2
set expandtab
set fileencodings=ucs-bom,utf-8,default,latin1
set history=50
set laststatus=2
set nomodeline
set runtimepath=~/.vim,~/.vim/plugged/coc.nvim,~/.vim/plugged/vim-code-dark,~/.vim/plugged/vim-airline,/usr/share/vim/vimfiles,/usr/share/vim/vim92,/usr/share/vim/vim92/pack/dist/opt/netrw,/usr/share/vim/vimfiles/after,~/.vim/after
set shiftwidth=2
set softtabstop=2
set suffixes=.bak,~,.o,.h,.info,.swp,.obj,.info,.aux,.log,.dvi,.bbl,.out,.o,.lo
set tabstop=2
set window=39
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
doautoall SessionLoadPre
silent only
silent tabonly
cd ~/MyProjects/FoxieFoxGame
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
set shortmess+=aoO
badd +10 src/main.rs
badd +38 src/util/idented_writer.rs
badd +1 src/rendering/mod.rs
badd +1 ~/.vimr
badd +42 ~/.vimrc
badd +40 ~/.vimrc~
badd +330 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-types-29.0.3/src/instance.rs
badd +2 src/util/prefix_err.rs
badd +62 src/util/mod.rs
badd +579 ~/.rustup/toolchains/nightly-x86_64-unknown-linux-musl/lib/rustlib/src/rust/library/core/src/fmt/mod.rs
badd +230 ~/.rustup/toolchains/nightly-x86_64-unknown-linux-musl/lib/rustlib/src/rust/library/std/src/backtrace.rs
badd +105 src/logging/mod.rs
badd +0 src/window.rs
argglobal
%argdel
$argadd ~/MyProjects/FoxieFoxGame
set stal=2
tabnew +setlocal\ bufhidden=wipe
tabrewind
edit src/main.rs
argglobal
balt src/util/idented_writer.rs
onoremap <buffer> <silent> [[ :call rust#Jump('o', 'Back')
xnoremap <buffer> <silent> [[ :call rust#Jump('v', 'Back')
nnoremap <buffer> <silent> [[ :call rust#Jump('n', 'Back')
onoremap <buffer> <silent> ]] :call rust#Jump('o', 'Forward')
xnoremap <buffer> <silent> ]] :call rust#Jump('v', 'Forward')
nnoremap <buffer> <silent> ]] :call rust#Jump('n', 'Forward')
setlocal keymap=
setlocal noarabic
setlocal autoindent
setlocal backupcopy=
setlocal balloonexpr=
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=
setlocal bufhidden=
setlocal buflisted
setlocal buftype=
setlocal cindent
setlocal cinkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal cinoptions=L0,(s,Ws,J1,j1,m1
setlocal cinscopedecls=public,protected,private
setlocal cinwords=for,if,else,while,loop,impl,mod,unsafe,trait,struct,enum,fn,extern,macro
setlocal colorcolumn=
setlocal comments=s0:/*!,ex:*/,s1:/*,mb:*,ex:*/,:///,://!,://
setlocal commentstring=//\ %s
setlocal complete=.,w,b,u,t,i
setlocal completefunc=
setlocal completeopt=
setlocal concealcursor=
setlocal conceallevel=0
setlocal nocopyindent
setlocal cryptmethod=
setlocal nocursorbind
setlocal nocursorcolumn
setlocal nocursorline
setlocal cursorlineopt=both
setlocal define=
setlocal dictionary=
setlocal nodiff
setlocal diffanchors=
setlocal equalprg=
setlocal errorformat=%-G,%-Gerror:\ aborting\ %.%#,%-Gerror:\ Could\ not\ compile\ %.%#,%Eerror:\ %m,%Eerror[E%n]:\ %m,%Wwarning:\ %m,%Wwarning[E%n]:\ %m,%Inote:\ %m,%C\ %#-->\ %f:%l:%c,%C\ %#╭▸\ %f:%l:%c,%E\ \ left:%m,%C\ right:%m\ %f:%l:%c,%Z,%f:%l:%c:\ %t%*[^:]:\ %m,%f:%l:%c:\ %*\\d:%*\\d\ %t%*[^:]:\ %m,%-G%f:%l\ %s,%-G%*[\ ]^,%-G%*[\ ]^%*[~],%-G%*[\ ]...,%-G%\\s%#Downloading%.%#,%-G%\\s%#Checking%.%#,%-G%\\s%#Compiling%.%#,%-G%\\s%#Finished%.%#,%-G%\\s%#error:\ Could\ not\ compile\ %.%#,%-G%\\s%#To\ learn\ more\\,%.%#,%-G%\\s%#For\ more\ information\ about\ this\ error\\,%.%#,%-Gnote:\ Run\ with\ `RUST_BACKTRACE=%.%#,%.%#panicked\ at\ \\'%m\\'\\,\ %f:%l:%c
setlocal eventignorewin=
setlocal expandtab
if &filetype != 'rust'
setlocal filetype=rust
endif
setlocal fillchars=
setlocal findfunc=
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldmarker={{{,}}}
setlocal foldmethod=manual
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal formatoptions=croqnlj
setlocal formatprg=
setlocal grepformat=
setlocal grepprg=
setlocal iminsert=0
setlocal imsearch=-1
setlocal include=\\v^\\s*(pub\\s+)?use\\s+\\zs(\\f|:)+
setlocal includeexpr=rust#IncludeExpr(v:fname)
setlocal indentexpr=GetRustIndent(v:lnum)
setlocal indentkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal keywordprg=
setlocal lhistory=10
setlocal nolinebreak
setlocal nolisp
setlocal lispoptions=
setlocal lispwords=
setlocal nolist
setlocal listchars=
setlocal makeencoding=
setlocal makeprg=cargo\ $*
setlocal matchpairs=(:),{:},[:],<:>
setlocal nomodeline
setlocal modifiable
setlocal nrformats=bin,octal,hex
set number
setlocal number
setlocal numberwidth=4
setlocal omnifunc=
setlocal path=
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
set relativenumber
setlocal relativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal noscrollbind
setlocal scrolloff=-1
setlocal scrolloffpad=-1
setlocal shiftwidth=4
setlocal noshortname
setlocal showbreak=
setlocal sidescrolloff=-1
setlocal signcolumn=auto
setlocal smartindent
setlocal nosmoothscroll
setlocal softtabstop=4
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\	\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal spelloptions=
setlocal statusline=%!airline#statusline(1)
setlocal statuslineopt=
setlocal suffixesadd=.rs
setlocal noswapfile
setlocal synmaxcol=3000
if &syntax != 'rust'
setlocal syntax=rust
endif
setlocal tabstop=2
setlocal tagcase=
setlocal tagfunc=
setlocal tags=
setlocal termwinkey=
setlocal termwinscroll=10000
setlocal termwinsize=
setlocal textwidth=100
setlocal thesaurus=
setlocal thesaurusfunc=
setlocal noundofile
setlocal undolevels=-123456
setlocal varsofttabstop=
setlocal vartabstop=
setlocal virtualedit=
setlocal wincolor=
setlocal nowinfixbuf
setlocal nowinfixheight
setlocal nowinfixwidth
setlocal winhighlight=
set nowrap
setlocal nowrap
setlocal wrapmargin=0
silent! normal! zE
let &fdl = &fdl
let s:l = 3 - ((2 * winheight(0) + 18) / 37)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 3
normal! 0
lcd ~/MyProjects/FoxieFoxGame
tabnext
edit ~/MyProjects/FoxieFoxGame/src/window.rs
argglobal
onoremap <buffer> <silent> [[ :call rust#Jump('o', 'Back')
xnoremap <buffer> <silent> [[ :call rust#Jump('v', 'Back')
nnoremap <buffer> <silent> [[ :call rust#Jump('n', 'Back')
onoremap <buffer> <silent> ]] :call rust#Jump('o', 'Forward')
xnoremap <buffer> <silent> ]] :call rust#Jump('v', 'Forward')
nnoremap <buffer> <silent> ]] :call rust#Jump('n', 'Forward')
setlocal keymap=
setlocal noarabic
setlocal autoindent
setlocal backupcopy=
setlocal balloonexpr=
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=
setlocal bufhidden=
setlocal buflisted
setlocal buftype=
setlocal cindent
setlocal cinkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal cinoptions=L0,(s,Ws,J1,j1,m1
setlocal cinscopedecls=public,protected,private
setlocal cinwords=for,if,else,while,loop,impl,mod,unsafe,trait,struct,enum,fn,extern,macro
setlocal colorcolumn=
setlocal comments=s0:/*!,ex:*/,s1:/*,mb:*,ex:*/,:///,://!,://
setlocal commentstring=//\ %s
setlocal complete=.,w,b,u,t,i
setlocal completefunc=
setlocal completeopt=
setlocal concealcursor=
setlocal conceallevel=0
setlocal nocopyindent
setlocal cryptmethod=
setlocal nocursorbind
setlocal nocursorcolumn
setlocal nocursorline
setlocal cursorlineopt=both
setlocal define=
setlocal dictionary=
setlocal nodiff
setlocal diffanchors=
setlocal equalprg=
setlocal errorformat=%-G,%-Gerror:\ aborting\ %.%#,%-Gerror:\ Could\ not\ compile\ %.%#,%Eerror:\ %m,%Eerror[E%n]:\ %m,%Wwarning:\ %m,%Wwarning[E%n]:\ %m,%Inote:\ %m,%C\ %#-->\ %f:%l:%c,%C\ %#╭▸\ %f:%l:%c,%E\ \ left:%m,%C\ right:%m\ %f:%l:%c,%Z,%f:%l:%c:\ %t%*[^:]:\ %m,%f:%l:%c:\ %*\\d:%*\\d\ %t%*[^:]:\ %m,%-G%f:%l\ %s,%-G%*[\ ]^,%-G%*[\ ]^%*[~],%-G%*[\ ]...,%-G%\\s%#Downloading%.%#,%-G%\\s%#Checking%.%#,%-G%\\s%#Compiling%.%#,%-G%\\s%#Finished%.%#,%-G%\\s%#error:\ Could\ not\ compile\ %.%#,%-G%\\s%#To\ learn\ more\\,%.%#,%-G%\\s%#For\ more\ information\ about\ this\ error\\,%.%#,%-Gnote:\ Run\ with\ `RUST_BACKTRACE=%.%#,%.%#panicked\ at\ \\'%m\\'\\,\ %f:%l:%c
setlocal eventignorewin=
setlocal expandtab
if &filetype != 'rust'
setlocal filetype=rust
endif
setlocal fillchars=
setlocal findfunc=
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldmarker={{{,}}}
setlocal foldmethod=manual
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal formatoptions=croqnlj
setlocal formatprg=
setlocal grepformat=
setlocal grepprg=
setlocal iminsert=0
setlocal imsearch=-1
setlocal include=\\v^\\s*(pub\\s+)?use\\s+\\zs(\\f|:)+
setlocal includeexpr=rust#IncludeExpr(v:fname)
setlocal indentexpr=GetRustIndent(v:lnum)
setlocal indentkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal keywordprg=
setlocal lhistory=10
setlocal nolinebreak
setlocal nolisp
setlocal lispoptions=
setlocal lispwords=
setlocal nolist
setlocal listchars=
setlocal makeencoding=
setlocal makeprg=cargo\ $*
setlocal matchpairs=(:),{:},[:],<:>
setlocal nomodeline
setlocal modifiable
setlocal nrformats=bin,octal,hex
set number
setlocal number
setlocal numberwidth=4
setlocal omnifunc=
setlocal path=
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
set relativenumber
setlocal relativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal noscrollbind
setlocal scrolloff=-1
setlocal scrolloffpad=-1
setlocal shiftwidth=4
setlocal noshortname
setlocal showbreak=
setlocal sidescrolloff=-1
setlocal signcolumn=auto
setlocal smartindent
setlocal nosmoothscroll
setlocal softtabstop=4
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\	\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal spelloptions=
setlocal statusline=%!airline#statusline(1)
setlocal statuslineopt=
setlocal suffixesadd=.rs
setlocal swapfile
setlocal synmaxcol=3000
if &syntax != 'rust'
setlocal syntax=rust
endif
setlocal tabstop=2
setlocal tagcase=
setlocal tagfunc=
setlocal tags=
setlocal termwinkey=
setlocal termwinscroll=10000
setlocal termwinsize=
setlocal textwidth=100
setlocal thesaurus=
setlocal thesaurusfunc=
setlocal noundofile
setlocal undolevels=-123456
setlocal varsofttabstop=
setlocal vartabstop=
setlocal virtualedit=
setlocal wincolor=
setlocal nowinfixbuf
setlocal nowinfixheight
setlocal nowinfixwidth
setlocal winhighlight=
set nowrap
setlocal nowrap
setlocal wrapmargin=0
silent! normal! zE
let &fdl = &fdl
let s:l = 4 - ((3 * winheight(0) + 18) / 37)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 4
normal! 028|
lcd ~/MyProjects/FoxieFoxGame
tabnext 2
set stal=1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
set shortmess=filnxtToOS
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :

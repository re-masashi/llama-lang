#include <iostream>

extern "C"{
	int hello_world(){
		std::cout << "Hello world!\n";
		return 0;
	}

	int next_int(){
		int input;
		std::cin >> input;
		return input;
	}

	int println(int n){
		std::cout << n << std::endl;
		return n;
	}
}